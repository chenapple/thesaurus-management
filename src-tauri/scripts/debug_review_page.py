#!/usr/bin/env python3
"""
测试不同亚马逊站点的评论页面访问
"""
import asyncio
from playwright.async_api import async_playwright

async def test_review_access():
    # 测试不同站点
    test_cases = [
        {"country": "US", "asin": "B07X8Z8N7C", "url": "https://www.amazon.com/product-reviews/B07X8Z8N7C"},
        {"country": "DE", "asin": "B0C36HWY5T", "url": "https://www.amazon.de/product-reviews/B0C36HWY5T"},
        {"country": "UK", "asin": "B0C36HWY5T", "url": "https://www.amazon.co.uk/product-reviews/B0C36HWY5T"},
    ]

    async with async_playwright() as p:
        browser = await p.chromium.launch(
            headless=True,
            args=["--headless=new"]
        )

        for case in test_cases:
            print(f"\n{'='*60}")
            print(f"测试 {case['country']} 站: {case['url']}")
            print('='*60)

            context = await browser.new_context(
                user_agent='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
                locale='en-US' if case['country'] == 'US' else 'en-GB',
                viewport={'width': 1920, 'height': 1080}
            )
            page = await context.new_page()

            try:
                await page.goto(case['url'], wait_until='domcontentloaded', timeout=30000)
                await page.wait_for_timeout(3000)

                # 处理验证页面
                html = await page.content()
                continue_btns = ["Continue shopping", "Continuer les achats", "Weiter einkaufen"]
                for btn_text in continue_btns:
                    if btn_text in html:
                        print(f">>> 点击: {btn_text}")
                        btn = page.get_by_role("button", name=btn_text)
                        if await btn.count() > 0:
                            await btn.first.click()
                            await page.wait_for_timeout(3000)
                            break

                final_url = page.url
                title = await page.title()
                print(f"最终 URL: {final_url[:80]}...")
                print(f"页面标题: {title[:50]}...")

                # 检查是否为登录页面
                if '/ap/signin' in final_url or 'signin' in final_url.lower():
                    print("❌ 需要登录")
                else:
                    # 检查评论数量
                    review_count = await page.locator('[data-hook="review"]').count()
                    print(f"✓ 找到评论数: {review_count}")

                    if review_count > 0:
                        # 尝试提取一条评论
                        review = page.locator('[data-hook="review"]').first
                        body = review.locator('[data-hook="review-body"] span').first
                        if await body.count() > 0:
                            text = await body.inner_text()
                            print(f"  示例: {text[:80]}...")

            except Exception as e:
                print(f"❌ 错误: {e}")

            await context.close()

        await browser.close()

if __name__ == "__main__":
    asyncio.run(test_review_access())
