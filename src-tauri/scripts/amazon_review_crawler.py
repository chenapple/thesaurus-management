#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Amazon 评论爬虫 (Playwright 版本)
从产品页面提取评论（无需登录）

注意: 亚马逊评论页面(/product-reviews/)需要登录
因此改为从产品页面提取可见评论（约10-15条高质量评论）
"""

import sys
import json
import re
import asyncio
from datetime import datetime, timezone

try:
    from playwright.async_api import async_playwright
except ImportError:
    print(json.dumps({
        "error": "缺少 Playwright. 请运行: pip install playwright && playwright install chromium"
    }))
    sys.exit(1)

# 站点配置
COUNTRY_CONFIG = {
    "US": {"base_url": "https://www.amazon.com", "language": "en-US"},
    "UK": {"base_url": "https://www.amazon.co.uk", "language": "en-GB"},
    "DE": {"base_url": "https://www.amazon.de", "language": "de-DE"},
    "FR": {"base_url": "https://www.amazon.fr", "language": "fr-FR"},
    "IT": {"base_url": "https://www.amazon.it", "language": "it-IT"},
    "ES": {"base_url": "https://www.amazon.es", "language": "es-ES"},
    "JP": {"base_url": "https://www.amazon.co.jp", "language": "ja-JP"},
    "CA": {"base_url": "https://www.amazon.ca", "language": "en-CA"},
    "AU": {"base_url": "https://www.amazon.com.au", "language": "en-AU"},
}

# 继续购物按钮文本（各语言）
CONTINUE_BUTTONS = [
    "Continue shopping",
    "Continuer les achats",
    "Weiter einkaufen",
    "Continua gli acquisti",
    "Continuar comprando",
    "買い物を続ける",
]


def parse_star_rating(text: str) -> int:
    """从星级文本中提取数字 (如 '4,0 sur 5 étoiles' -> 4)"""
    if not text:
        return 0
    # 匹配 4.0 或 4,0 或 4
    match = re.search(r'(\d)[,.]?\d?\s*(out of|sur|von|su|de|of)', text, re.IGNORECASE)
    if match:
        return int(match.group(1))
    # 备选：直接匹配数字
    match = re.search(r'^(\d)', text)
    if match:
        return int(match.group(1))
    return 0


def parse_helpful_votes(text: str) -> int:
    """从有用投票文本中提取数字"""
    if not text:
        return 0
    match = re.search(r'(\d+)', text)
    return int(match.group(1)) if match else 0


async def fetch_reviews_from_product_page(asin: str, country: str, headless="new") -> dict:
    """
    从产品页面获取评论

    返回:
    {
        "asin": str,
        "country": str,
        "reviews": [...],
        "summary": {
            "total": int,
            "by_star": {1: int, 2: int, 3: int, 4: int, 5: int}
        },
        "fetched_at": str,
        "error": str | null
    }
    """
    config = COUNTRY_CONFIG.get(country.upper(), COUNTRY_CONFIG["US"])

    result = {
        "asin": asin,
        "country": country,
        "reviews": [],
        "summary": {
            "total": 0,
            "by_star": {1: 0, 2: 0, 3: 0, 4: 0, 5: 0}
        },
        "fetched_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "error": None
    }

    async with async_playwright() as p:
        # 配置浏览器
        launch_options = {}
        if headless == "new":
            launch_options["headless"] = True
            launch_options["args"] = ["--headless=new"]
        elif headless is False:
            launch_options["headless"] = False
        else:
            launch_options["headless"] = headless

        browser = await p.chromium.launch(**launch_options)

        # 时区映射
        timezones = {
            "DE": "Europe/Berlin", "FR": "Europe/Paris", "UK": "Europe/London",
            "IT": "Europe/Rome", "ES": "Europe/Madrid", "US": "America/New_York",
            "JP": "Asia/Tokyo", "CA": "America/Toronto", "AU": "Australia/Sydney",
        }

        context = await browser.new_context(
            locale=config['language'],
            timezone_id=timezones.get(country.upper(), "America/New_York"),
            viewport={"width": 1920, "height": 1080},
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        )

        page = await context.new_page()
        product_url = f"{config['base_url']}/dp/{asin}"

        try:
            print(f"[DEBUG] 访问产品页: {product_url}", file=sys.stderr)
            await page.goto(product_url, wait_until="domcontentloaded", timeout=30000)
            await page.wait_for_timeout(3000)

            # 处理验证/Cookie 页面
            html = await page.content()
            for btn_text in CONTINUE_BUTTONS:
                if btn_text in html:
                    print(f"[DEBUG] 点击: {btn_text}", file=sys.stderr)
                    btn = page.get_by_role("button", name=btn_text)
                    if await btn.count() > 0:
                        await btn.first.click()
                        await page.wait_for_timeout(3000)
                        break

            # 处理 Cookie 同意弹窗
            try:
                cookie_btn = page.locator('#sp-cc-accept')
                if await cookie_btn.is_visible(timeout=1500):
                    await cookie_btn.click()
                    await page.wait_for_timeout(1000)
            except:
                pass

            # 滚动页面加载评论区域
            print("[DEBUG] 滚动页面加载评论...", file=sys.stderr)
            for i in range(6):
                await page.evaluate(f'window.scrollTo(0, {i * 1200})')
                await page.wait_for_timeout(400)

            # 查找评论容器
            review_containers = page.locator('[data-hook="review"]')
            review_count = await review_containers.count()

            print(f"[DEBUG] 找到 {review_count} 条评论", file=sys.stderr)

            # 提取每条评论
            for i in range(review_count):
                review_elem = review_containers.nth(i)

                try:
                    review_data = {
                        "star_rating": 0,
                        "review_text": "",
                        "review_title": "",
                        "review_date": None,
                        "helpful_votes": 0,
                    }

                    # 提取星级
                    try:
                        star_elem = review_elem.locator('[data-hook="review-star-rating"], [data-hook="cmps-review-star-rating"]').first
                        if await star_elem.count() > 0:
                            star_text = await star_elem.inner_text()
                            review_data["star_rating"] = parse_star_rating(star_text)
                    except:
                        pass

                    # 提取标题
                    try:
                        title_elem = review_elem.locator('[data-hook="review-title"] span:not(.a-icon-alt)').first
                        if await title_elem.count() > 0:
                            title = await title_elem.inner_text()
                            review_data["review_title"] = title.strip() if title else ""
                    except:
                        pass

                    # 提取评论文本
                    try:
                        text_elem = review_elem.locator('[data-hook="review-body"] span').first
                        if await text_elem.count() > 0:
                            text = await text_elem.inner_text()
                            review_data["review_text"] = text.strip() if text else ""
                    except:
                        pass

                    # 提取日期
                    try:
                        date_elem = review_elem.locator('[data-hook="review-date"]').first
                        if await date_elem.count() > 0:
                            date_text = await date_elem.inner_text()
                            review_data["review_date"] = date_text.strip() if date_text else None
                    except:
                        pass

                    # 提取有用投票数
                    try:
                        helpful_elem = review_elem.locator('[data-hook="helpful-vote-statement"]').first
                        if await helpful_elem.count() > 0:
                            helpful_text = await helpful_elem.inner_text()
                            review_data["helpful_votes"] = parse_helpful_votes(helpful_text)
                    except:
                        pass

                    # 只添加有内容的评论
                    if review_data["review_text"]:
                        result["reviews"].append(review_data)
                        star = review_data["star_rating"]
                        if 1 <= star <= 5:
                            result["summary"]["by_star"][star] += 1

                except Exception as e:
                    print(f"[DEBUG] 解析评论 {i+1} 出错: {e}", file=sys.stderr)
                    continue

            result["summary"]["total"] = len(result["reviews"])
            print(f"[DEBUG] 评论获取完成，共 {result['summary']['total']} 条", file=sys.stderr)

        except Exception as e:
            result["error"] = str(e)
            print(f"[DEBUG] 获取评论失败: {e}", file=sys.stderr)

        finally:
            await browser.close()

    return result


def main():
    if len(sys.argv) < 3:
        print(json.dumps({
            "error": "用法: python amazon_review_crawler.py <asin> <country> [headless]"
        }))
        sys.exit(1)

    asin = sys.argv[1]
    country = sys.argv[2].upper()
    headless_arg = sys.argv[3].lower() if len(sys.argv) > 3 else "new"

    if headless_arg == "new":
        headless = "new"
    elif headless_arg == "false":
        headless = False
    else:
        headless = True

    result = asyncio.run(fetch_reviews_from_product_page(asin, country, headless))

    # 输出完成标记
    complete = {
        "type": "complete",
        "result": result
    }
    output = json.dumps(complete, ensure_ascii=False)
    sys.stdout.buffer.write(output.encode('utf-8'))
    sys.stdout.buffer.write(b'\n')


if __name__ == "__main__":
    main()
