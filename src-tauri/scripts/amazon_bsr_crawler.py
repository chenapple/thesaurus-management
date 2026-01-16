#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Amazon Best Seller 排名爬虫 (Playwright 版本)
爬取指定类目的 Top 100 BSR 产品数据
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
    "US": {
        "base_url": "https://www.amazon.com",
        "bsr_path": "/gp/bestsellers",
        "language": "en-US",
    },
    "UK": {
        "base_url": "https://www.amazon.co.uk",
        "bsr_path": "/gp/bestsellers",
        "language": "en-GB",
    },
    "DE": {
        "base_url": "https://www.amazon.de",
        "bsr_path": "/gp/bestsellers",
        "language": "de-DE",
    },
    "FR": {
        "base_url": "https://www.amazon.fr",
        "bsr_path": "/gp/bestsellers",
        "language": "fr-FR",
    },
    "IT": {
        "base_url": "https://www.amazon.it",
        "bsr_path": "/gp/bestsellers",
        "language": "it-IT",
    },
    "ES": {
        "base_url": "https://www.amazon.es",
        "bsr_path": "/gp/bestsellers",
        "language": "es-ES",
    },
    "JP": {
        "base_url": "https://www.amazon.co.jp",
        "bsr_path": "/gp/bestsellers",
        "language": "ja-JP",
    },
    "CA": {
        "base_url": "https://www.amazon.ca",
        "bsr_path": "/gp/bestsellers",
        "language": "en-CA",
    },
    "MX": {
        "base_url": "https://www.amazon.com.mx",
        "bsr_path": "/gp/bestsellers",
        "language": "es-MX",
    },
    "BR": {
        "base_url": "https://www.amazon.com.br",
        "bsr_path": "/gp/bestsellers",
        "language": "pt-BR",
    },
    "AU": {
        "base_url": "https://www.amazon.com.au",
        "bsr_path": "/gp/bestsellers",
        "language": "en-AU",
    },
    "NL": {
        "base_url": "https://www.amazon.nl",
        "bsr_path": "/gp/bestsellers",
        "language": "nl-NL",
    },
    "SE": {
        "base_url": "https://www.amazon.se",
        "bsr_path": "/gp/bestsellers",
        "language": "sv-SE",
    },
    "PL": {
        "base_url": "https://www.amazon.pl",
        "bsr_path": "/gp/bestsellers",
        "language": "pl-PL",
    },
}


async def extract_product_data(item, rank: int) -> dict:
    """从单个产品卡片中提取数据"""
    product = {
        "rank": rank,
        "asin": None,
        "title": None,
        "price": None,
        "rating": None,
        "reviews": 0,
        "image_url": None,
        "in_stock": True,
    }

    try:
        # 方法1: 从 data-asin 属性提取 ASIN
        try:
            asin = await item.get_attribute('data-asin')
            if asin and len(asin) == 10:
                product["asin"] = asin
        except:
            pass

        # 方法2: 从链接中提取 ASIN
        if not product["asin"]:
            link_selectors = [
                'a[href*="/dp/"]',
                'a.a-link-normal[href*="/dp/"]',
                'a[href*="/gp/product/"]',
            ]
            for link_sel in link_selectors:
                try:
                    link = await item.locator(link_sel).first.get_attribute('href', timeout=2000)
                    if link:
                        asin_match = re.search(r'/dp/([A-Z0-9]{10})', link) or re.search(r'/product/([A-Z0-9]{10})', link)
                        if asin_match:
                            product["asin"] = asin_match.group(1)
                            break
                except:
                    continue

        # 提取标题
        title_elem = item.locator('.p13n-sc-truncate, .a-link-normal span, ._cDEzb_p13n-sc-css-line-clamp-3_g3dy1').first
        try:
            title = await title_elem.text_content(timeout=1000)
            if title:
                product["title"] = title.strip()[:200]  # 限制长度
        except:
            pass

        # 提取价格
        price_selectors = [
            '._cDEzb_p13n-sc-price_3mJ9Z',
            '.p13n-sc-price',
            '.a-price .a-offscreen',
            'span.a-price span',
        ]
        for sel in price_selectors:
            try:
                price_elem = item.locator(sel).first
                price = await price_elem.text_content(timeout=500)
                if price:
                    product["price"] = price.strip()
                    break
            except:
                continue

        # 提取评分
        rating_selectors = [
            'span.a-icon-alt',
            '.a-icon-star-small span',
            'i.a-icon-star span',
        ]
        for sel in rating_selectors:
            try:
                rating_elem = item.locator(sel).first
                rating_text = await rating_elem.text_content(timeout=500)
                if rating_text:
                    rating_match = re.search(r'([\d.,]+)', rating_text)
                    if rating_match:
                        product["rating"] = float(rating_match.group(1).replace(',', '.'))
                        break
            except:
                continue

        # 提取评论数 - 使用更精确的选择器
        # 评论数通常是一个链接，点击可以跳转到评论区，格式如 "5,511" 或 "5.511"
        review_selectors = [
            # BSR 页面专用选择器 - 评论数链接
            'a[href*="#customerReviews"]',
            'a[href*="reviews"]',
            # 评论数通常在评分旁边，是一个独立的 span
            '.a-icon-row span.a-size-small:not(.a-icon-alt)',
            'span[data-a-size="small"][aria-label*="avis"]',  # 法语
            'span[data-a-size="small"][aria-label*="review"]',  # 英语
            'span[data-a-size="small"][aria-label*="Bewertung"]',  # 德语
        ]
        for sel in review_selectors:
            try:
                review_elem = item.locator(sel).first
                review_text = await review_elem.text_content(timeout=500)
                if review_text:
                    # 清理文本，只保留数字和分隔符
                    review_text = review_text.strip()
                    # 跳过评分相关文本（包含 "star", "étoile", "Stern" 等）
                    if any(kw in review_text.lower() for kw in ['star', 'étoile', 'stern', 'stell', 'estrella', '星']):
                        continue
                    # 移除千位分隔符（逗号、点、空格）并提取数字
                    # 先检查是否是纯数字格式（可能带分隔符）
                    num_str = re.sub(r'[\s,.]', '', review_text)
                    # 确保是纯数字且长度合理（评论数通常是数字）
                    if num_str.isdigit() and len(num_str) >= 1:
                        product["reviews"] = int(num_str)
                        break
            except:
                continue

        # 如果上面没找到，尝试通过 aria-label 属性获取评论数
        if product["reviews"] == 0:
            try:
                # 查找带有评论数信息的元素
                aria_elem = item.locator('[aria-label*="rating"]').first
                aria_label = await aria_elem.get_attribute('aria-label', timeout=500)
                if aria_label:
                    # 尝试从 aria-label 中提取评论数，格式如 "4.5 out of 5 stars, 5,511 ratings"
                    review_match = re.search(r'([\d,.\s]+)\s*(?:ratings|reviews|avis|bewertung|recensioni|valoraciones)', aria_label, re.IGNORECASE)
                    if review_match:
                        num_str = re.sub(r'[\s,.]', '', review_match.group(1))
                        if num_str.isdigit():
                            product["reviews"] = int(num_str)
            except:
                pass

        # 提取图片 URL
        try:
            img_elem = item.locator('img').first
            img_url = await img_elem.get_attribute('src', timeout=500)
            if img_url:
                product["image_url"] = img_url
        except:
            pass

    except Exception as e:
        print(f"[DEBUG] 提取产品数据出错: {e}", file=sys.stderr)

    return product


async def fetch_bsr_page(page, url: str, start_rank: int = 1) -> list:
    """爬取单个 BSR 页面的产品"""
    products = []

    await page.goto(url, wait_until='domcontentloaded', timeout=30000)
    await page.wait_for_timeout(3000)  # 增加等待时间

    # 尝试滚动页面触发懒加载
    await page.evaluate('window.scrollTo(0, document.body.scrollHeight / 2)')
    await page.wait_for_timeout(1000)

    # 等待产品列表加载 - 优先使用能直接获取 ASIN 的选择器
    product_selectors = [
        '#gridItemRoot',
        '.zg-grid-general-faceout',
        'div[id^="p13n-asin-index-"]',  # 子类目页面的产品卡片
        'div[data-asin]:not([data-asin=""])',  # 有 data-asin 属性且不为空
        '.zg-item-immersion',
        '.zg-carousel-general-faceout',
        '.p13n-desktop-grid div[data-asin]',  # 更精确的子类目选择器
    ]

    items = None
    for selector in product_selectors:
        try:
            items = page.locator(selector)
            count = await items.count()
            if count > 1:  # 至少要有 2 个产品才认为是有效的选择器
                print(f"[DEBUG] 使用选择器 {selector} 找到 {count} 个产品", file=sys.stderr)
                break
            elif count == 1:
                print(f"[DEBUG] 选择器 {selector} 只找到 1 个元素，尝试下一个选择器", file=sys.stderr)
                continue
        except:
            continue

    if items is None:
        print(f"[DEBUG] 未找到产品列表", file=sys.stderr)
        return products

    count = await items.count()
    for i in range(min(count, 50)):  # 每页最多 50 个
        try:
            item = items.nth(i)
            rank = start_rank + i
            product = await extract_product_data(item, rank)
            if product["asin"]:  # 只添加成功提取到 ASIN 的产品
                products.append(product)
        except Exception as e:
            print(f"[DEBUG] 处理第 {i+1} 个产品出错: {e}", file=sys.stderr)
            continue

    return products


async def crawl_bsr(marketplace: str, category_id: str, headless: bool = True) -> dict:
    """主爬虫函数"""

    config = COUNTRY_CONFIG.get(marketplace.upper())
    if not config:
        return {
            "error": f"不支持的站点: {marketplace}",
            "marketplace": marketplace,
            "category_id": category_id,
            "products": [],
            "snapshot_date": datetime.now(timezone.utc).isoformat(),
        }

    result = {
        "marketplace": marketplace.upper(),
        "category_id": category_id,
        "products": [],
        "snapshot_date": datetime.now(timezone.utc).isoformat(),
        "error": None,
    }

    async with async_playwright() as p:
        # 启动浏览器
        browser = await p.chromium.launch(
            headless=headless,
            args=[
                '--disable-blink-features=AutomationControlled',
                '--disable-dev-shm-usage',
                '--no-sandbox',
            ]
        )

        context = await browser.new_context(
            viewport={'width': 1920, 'height': 1080},
            user_agent='Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            locale=config['language'],
        )

        # 设置反检测脚本
        await context.add_init_script("""
            Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
            Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
        """)

        page = await context.new_page()

        try:
            # 构建 BSR URL
            # 格式: /gp/bestsellers/{category_id} 或 /Best-Sellers/zgbs/{category_id}
            base_url = config['base_url']

            # 第一页
            bsr_url_1 = f"{base_url}/gp/bestsellers/{category_id}"
            print(f"[DEBUG] 正在爬取第一页: {bsr_url_1}", file=sys.stderr)

            page1_products = await fetch_bsr_page(page, bsr_url_1, start_rank=1)
            result["products"].extend(page1_products)
            print(f"[DEBUG] 第一页获取到 {len(page1_products)} 个产品", file=sys.stderr)

            # 第二页（如果有的话）
            await page.wait_for_timeout(2000)

            # 尝试查找并点击第二页
            page2_selectors = [
                'a[href*="pg=2"]',
                'li.a-last a',
                '.a-pagination li:nth-child(2) a',
                'a:has-text("2")',
            ]

            page2_clicked = False
            for selector in page2_selectors:
                try:
                    page2_link = page.locator(selector).first
                    if await page2_link.is_visible(timeout=2000):
                        await page2_link.click()
                        await page.wait_for_timeout(3000)
                        page2_clicked = True
                        break
                except:
                    continue

            if page2_clicked:
                page2_products = await fetch_bsr_page(page, page.url, start_rank=51)
                result["products"].extend(page2_products)
                print(f"[DEBUG] 第二页获取到 {len(page2_products)} 个产品", file=sys.stderr)
            else:
                # 尝试直接访问第二页 URL
                bsr_url_2 = f"{base_url}/gp/bestsellers/{category_id}/ref=zg_bs_pg_2?pg=2"
                print(f"[DEBUG] 尝试直接访问第二页: {bsr_url_2}", file=sys.stderr)
                try:
                    page2_products = await fetch_bsr_page(page, bsr_url_2, start_rank=51)
                    result["products"].extend(page2_products)
                    print(f"[DEBUG] 第二页获取到 {len(page2_products)} 个产品", file=sys.stderr)
                except Exception as e:
                    print(f"[DEBUG] 第二页访问失败: {e}", file=sys.stderr)

            print(f"[DEBUG] 共获取到 {len(result['products'])} 个产品", file=sys.stderr)

        except Exception as e:
            result["error"] = str(e)
            print(f"[ERROR] 爬取出错: {e}", file=sys.stderr)

        finally:
            await browser.close()

    return result


async def handle_continue_shopping_button(page) -> bool:
    """处理 Amazon 的"继续购物"拦截页面"""
    continue_texts = [
        "Continuer les achats",   # 法语
        "Continue shopping",       # 英语
        "Weiter einkaufen",        # 德语
        "Weiter shoppen",          # 德语变体
        "Continua lo shopping",    # 意大利语
        "Continua con gli acquisti",  # 意大利语变体
        "Seguir comprando",        # 西班牙语
        "ショッピングを続ける",      # 日语
        "继续购物",                 # 中文
    ]

    try:
        # 方法1: 使用 get_by_role 查找按钮
        for text in continue_texts:
            try:
                btn = page.get_by_role("button", name=text)
                if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                    print(f"[DEBUG] 发现继续购物按钮 (role): {text}", file=sys.stderr)
                    await btn.first.click()
                    await page.wait_for_timeout(2000)
                    return True
            except:
                pass

            try:
                btn = page.get_by_text(text, exact=True)
                if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                    print(f"[DEBUG] 发现继续购物按钮 (text): {text}", file=sys.stderr)
                    await btn.first.click()
                    await page.wait_for_timeout(2000)
                    return True
            except:
                pass

        # 方法2: 使用 CSS 选择器
        css_selectors = [
            'input[type="submit"][value*="Continuer"]',
            'input[type="submit"][value*="Continue"]',
            'input[type="submit"][value*="Weiter"]',
            'input[type="submit"][value*="Continua"]',
            'input[type="submit"][value*="Seguir"]',
            '.a-button-primary input[type="submit"]',
            'span.a-button-inner input[type="submit"]',
        ]
        for selector in css_selectors:
            try:
                elements = page.locator(selector)
                count = await elements.count()
                for i in range(count):
                    elem = elements.nth(i)
                    if await elem.is_visible(timeout=500):
                        text = await elem.get_attribute('value') or ''
                        if any(kw in text for kw in ['Continuer', 'Continue', 'Weiter', 'Continua', 'Seguir']):
                            print(f"[DEBUG] 发现继续购物按钮 (css): {text}", file=sys.stderr)
                            await elem.click()
                            await page.wait_for_timeout(2000)
                            return True
            except:
                pass

    except Exception as e:
        print(f"[DEBUG] 处理继续购物按钮出错: {e}", file=sys.stderr)

    return False


async def discover_subcategories(marketplace: str, parent_category: str, headless: bool = True) -> dict:
    """发现子类目

    Amazon 类目系统说明:
    - 类目 ID 不是按层级嵌套的（子类目 ID 不包含父类目 ID）
    - 例如: beauty (一级) -> beauty/211005031 (Manucure) -> beauty/3055875031 (Soins)
    - 需要通过 DOM 结构来识别哪些是当前类目的直接子类目

    策略:
    1. 找到侧边栏中当前选中的类目元素
    2. 提取该元素下方缩进显示的子类目链接
    3. 如果找不到选中元素，回退到排除法（排除祖先类目）
    """
    config = COUNTRY_CONFIG.get(marketplace.upper())
    if not config:
        return {
            "error": f"不支持的站点: {marketplace}",
            "marketplace": marketplace,
            "parent_category": parent_category,
            "subcategories": [],
        }

    result = {
        "marketplace": marketplace.upper(),
        "parent_category": parent_category,
        "subcategories": [],
        "error": None,
    }

    parent_parts = parent_category.split("/")
    parent_base = parent_parts[0]  # 一级类目名称
    parent_id = parent_parts[-1] if len(parent_parts) > 1 else None  # 当前类目数字ID

    print(f"[DEBUG] 父类目: {parent_category}, 基础: {parent_base}, ID: {parent_id}", file=sys.stderr)

    async with async_playwright() as p:
        browser = await p.chromium.launch(
            headless=headless,
            args=[
                '--disable-blink-features=AutomationControlled',
                '--disable-dev-shm-usage',
                '--no-sandbox',
            ]
        )

        context = await browser.new_context(
            viewport={'width': 1920, 'height': 1080},
            user_agent='Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            locale=config['language'],
        )

        await context.add_init_script("""
            Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
        """)

        page = await context.new_page()

        try:
            base_url = config['base_url']
            url = f"{base_url}/gp/bestsellers/{parent_category}"
            print(f"[DEBUG] 正在发现子类目: {url}", file=sys.stderr)

            await page.goto(url, wait_until='domcontentloaded', timeout=30000)
            await page.wait_for_timeout(3000)

            await handle_continue_shopping_button(page)

            await page.evaluate("window.scrollTo(0, 500)")
            await page.wait_for_timeout(1000)

            # 方法1: 尝试通过 DOM 结构找子类目
            # Amazon 的侧边栏结构：嵌套的 div[role="group"]
            # 当前选中的类目通常有特殊样式（粗体、不同颜色等）
            subcategories_found = False

            # 尝试找到当前类目后面的嵌套 group
            try:
                # 使用 JavaScript 来分析 DOM 结构
                subcats = await page.evaluate('''(parentId) => {
                    const results = [];

                    // 查找侧边栏
                    const sidebar = document.querySelector('[class*="zg-browse"]') ||
                                  document.querySelector('div[role="group"]');
                    if (!sidebar) return results;

                    // 查找所有链接
                    const allLinks = sidebar.querySelectorAll('a');
                    let foundParent = false;
                    let parentIndent = -1;

                    for (const link of allLinks) {
                        const href = link.getAttribute('href') || '';
                        const text = link.textContent?.trim() || '';

                        // 检查是否是当前父类目
                        if (parentId && (href.includes('/' + parentId) || href.endsWith('/' + parentId.split('/').pop()))) {
                            // 计算当前元素的缩进级别
                            let el = link.parentElement;
                            let indent = 0;
                            while (el && el !== sidebar) {
                                if (el.getAttribute('role') === 'group' || el.tagName === 'UL') indent++;
                                el = el.parentElement;
                            }
                            parentIndent = indent;
                            foundParent = true;
                            continue;
                        }

                        // 如果已经找到父类目，收集它后面同级或更深的链接
                        if (foundParent) {
                            // 计算当前元素的缩进级别
                            let el = link.parentElement;
                            let indent = 0;
                            while (el && el !== sidebar) {
                                if (el.getAttribute('role') === 'group' || el.tagName === 'UL') indent++;
                                el = el.parentElement;
                            }

                            // 只要比父类目缩进更深的就是子类目
                            if (indent > parentIndent) {
                                // 提取类目 ID
                                const zgbsMatch = href.match(/\\/zgbs\\/([^/]+(?:\\/\\d+)*)/);
                                const bsMatch = href.match(/\\/bestsellers\\/([^/]+(?:\\/\\d+)*)/);
                                const catId = (zgbsMatch || bsMatch)?.[1];

                                if (catId && text && !results.some(r => r.category_id === catId)) {
                                    results.push({
                                        name: text,
                                        category_id: catId,
                                        url: href.startsWith('http') ? href : window.location.origin + href,
                                        indent: indent
                                    });
                                }
                            } else if (indent <= parentIndent && results.length > 0) {
                                // 已经回到同级或更高级别，停止收集
                                break;
                            }
                        }
                    }

                    // 过滤：只保留第一层子类目（最小缩进级别的）
                    if (results.length > 0) {
                        const minIndent = Math.min(...results.map(r => r.indent));
                        return results.filter(r => r.indent === minIndent).map(r => ({
                            name: r.name,
                            category_id: r.category_id,
                            url: r.url
                        }));
                    }

                    return results;
                }''', parent_category)

                if subcats and len(subcats) > 0:
                    result["subcategories"] = subcats
                    subcategories_found = True
                    print(f"[DEBUG] 通过 DOM 结构找到 {len(subcats)} 个子类目", file=sys.stderr)
                    for s in subcats:
                        print(f"[DEBUG]   - {s['name']} -> {s['category_id']}", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] DOM 方法失败: {e}", file=sys.stderr)

            # 方法2: 回退到排除法
            if not subcategories_found:
                print("[DEBUG] 回退到排除法", file=sys.stderr)

                # 收集所有类目链接
                all_candidates = []
                link_selectors = [
                    f'a[href*="/zgbs/{parent_base}/"]',
                    f'a[href*="/bestsellers/{parent_base}/"]',
                ]

                for selector in link_selectors:
                    try:
                        links = page.locator(selector)
                        count = await links.count()
                        for i in range(min(count, 100)):
                            try:
                                link = links.nth(i)
                                href = await link.get_attribute('href', timeout=2000)
                                name = await link.text_content(timeout=2000)

                                if not href or not name:
                                    continue
                                name = name.strip()
                                if not name or len(name) > 100:
                                    continue

                                category_match = re.search(r'/zgbs/([^/]+(?:/\d+)*)', href) or \
                                                re.search(r'/bestsellers/([^/]+(?:/\d+)*)', href)
                                if category_match:
                                    category_id = category_match.group(1)
                                    all_candidates.append({
                                        "name": name,
                                        "category_id": category_id,
                                        "url": href if href.startswith('http') else f"{base_url}{href}",
                                    })
                            except:
                                continue
                    except:
                        continue

                print(f"[DEBUG] 排除法: 共找到 {len(all_candidates)} 个候选", file=sys.stderr)

                # 排除父类目和祖先类目
                # 祖先类目的特征：它们的 ID 是当前页面 URL 的前缀部分
                seen_ids = set()
                for candidate in all_candidates:
                    cid = candidate["category_id"]

                    if cid in seen_ids:
                        continue

                    # 跳过当前类目本身
                    if cid == parent_category:
                        print(f"[DEBUG]   排除当前类目: {candidate['name']} -> {cid}", file=sys.stderr)
                        continue

                    # 跳过一级类目（祖先）
                    if cid == parent_base:
                        print(f"[DEBUG]   排除祖先类目: {candidate['name']} -> {cid}", file=sys.stderr)
                        continue

                    # 跳过是当前类目祖先的类目
                    # 如果 parent_category 是 beauty/123/456，那么 beauty/123 是祖先
                    is_ancestor = False
                    if len(parent_parts) > 1:
                        # 检查候选是否是祖先
                        cid_parts = cid.split("/")
                        if len(cid_parts) < len(parent_parts):
                            # 检查是否是前缀
                            if parent_category.startswith(cid + "/") or parent_category.startswith(cid):
                                is_ancestor = True

                    if is_ancestor:
                        print(f"[DEBUG]   排除祖先类目: {candidate['name']} -> {cid}", file=sys.stderr)
                        continue

                    seen_ids.add(cid)
                    result["subcategories"].append(candidate)
                    print(f"[DEBUG]   保留子类目: {candidate['name']} -> {cid}", file=sys.stderr)

            print(f"[DEBUG] 最终找到 {len(result['subcategories'])} 个子类目", file=sys.stderr)

        except Exception as e:
            result["error"] = str(e)
            print(f"[ERROR] 发现子类目出错: {e}", file=sys.stderr)

        finally:
            await browser.close()

    return result


def main():
    """命令行入口"""
    if len(sys.argv) < 3:
        print(json.dumps({
            "error": "用法: python amazon_bsr_crawler.py <command> <marketplace> [args...]\n"
                    "命令:\n"
                    "  bsr <marketplace> <category_id> [headless] - 爬取 BSR 数据\n"
                    "  discover <marketplace> <parent_category> [headless] - 发现子类目"
        }))
        sys.exit(1)

    command = sys.argv[1]

    if command == "discover":
        if len(sys.argv) < 4:
            print(json.dumps({"error": "用法: python amazon_bsr_crawler.py discover <marketplace> <parent_category> [headless]"}))
            sys.exit(1)
        marketplace = sys.argv[2]
        parent_category = sys.argv[3]
        headless = sys.argv[4].lower() == 'true' if len(sys.argv) > 4 else True
        print(f"[DEBUG] 开始发现子类目: marketplace={marketplace}, parent={parent_category}, headless={headless}", file=sys.stderr)
        result = asyncio.run(discover_subcategories(marketplace, parent_category, headless))
        print(json.dumps(result, ensure_ascii=False))

    elif command == "bsr":
        if len(sys.argv) < 4:
            print(json.dumps({"error": "用法: python amazon_bsr_crawler.py bsr <marketplace> <category_id> [headless]"}))
            sys.exit(1)
        marketplace = sys.argv[2]
        category_id = sys.argv[3]
        headless = sys.argv[4].lower() == 'true' if len(sys.argv) > 4 else True
        print(f"[DEBUG] 开始爬取 BSR: marketplace={marketplace}, category={category_id}, headless={headless}", file=sys.stderr)
        result = asyncio.run(crawl_bsr(marketplace, category_id, headless))
        print(json.dumps(result, ensure_ascii=False))

    else:
        # 兼容旧的调用方式
        marketplace = sys.argv[1]
        category_id = sys.argv[2]
        headless = sys.argv[3].lower() == 'true' if len(sys.argv) > 3 else True
        print(f"[DEBUG] 开始爬取 BSR: marketplace={marketplace}, category={category_id}, headless={headless}", file=sys.stderr)
        result = asyncio.run(crawl_bsr(marketplace, category_id, headless))
        print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
