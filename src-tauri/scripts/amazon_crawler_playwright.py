#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Amazon 关键词排名爬虫 (Playwright 版本)
使用 Playwright 浏览器自动化，可以正确设置配送地址
"""

import sys
import json
import re
import asyncio
from datetime import datetime, timezone
from urllib.parse import quote_plus

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
        "language": "en-US",
        "zipcode": "10001",
        "market_param": "",
        "currency": "USD"
    },
    "UK": {
        "base_url": "https://www.amazon.co.uk",
        "language": "en-GB",
        "zipcode": "SW1A 1AA",
        "market_param": "",
        "currency": "GBP"
    },
    "DE": {
        "base_url": "https://www.amazon.de",
        "language": "de-DE",
        "zipcode": "10115",
        "market_param": "__mk_de_DE=%C3%85M%C3%85%C5%BD%C3%95%C3%91",
        "currency": "EUR"
    },
    "FR": {
        "base_url": "https://www.amazon.fr",
        "language": "fr-FR",
        "zipcode": "75001",
        "market_param": "__mk_fr_FR=%C3%85M%C3%85%C5%BD%C3%95%C3%91",
        "currency": "EUR"
    },
    "IT": {
        "base_url": "https://www.amazon.it",
        "language": "it-IT",
        "zipcode": "00100",
        "market_param": "__mk_it_IT=%C3%85M%C3%85%C5%BD%C3%95%C3%91",
        "currency": "EUR"
    },
    "ES": {
        "base_url": "https://www.amazon.es",
        "language": "es-ES",
        "zipcode": "28001",
        "market_param": "__mk_es_ES=%C3%85M%C3%85%C5%BD%C3%95%C3%91",
        "currency": "EUR"
    }
}

# 期望的配送地址显示模式（用于验证邮编是否设置成功）
EXPECTED_ADDRESS_KEYWORDS = {
    'DE': ['Deutschland', 'Germany', '10115', 'Berlin', 'Deutsch'],
    'FR': ['France', 'Frankreich', '75001', 'Paris'],
    'UK': ['United Kingdom', 'UK', 'GB', 'SW1A', 'London', 'Britain'],
    'IT': ['Italia', 'Italy', 'Italien', '00100', 'Roma', 'Rom'],
    'ES': ['España', 'Spain', 'Spanien', '28001', 'Madrid'],
    'US': ['United States', 'USA', '10001', 'New York'],
}


async def set_delivery_address(page, country: str, zipcode: str, max_retries: int = 5) -> tuple[bool, str]:
    """
    设置配送地址 - 基于用户确认的真实操作流程

    返回: (是否成功, 当前地址文本)
    """

    for attempt in range(max_retries):
        print(f"[DEBUG] 尝试设置邮编 (第{attempt + 1}次)...", file=sys.stderr)

        # 先检查当前地址是否已经正确
        try:
            current_addr = await page.locator('#glow-ingress-line2').text_content(timeout=2000)
            current_addr = current_addr.strip() if current_addr else ""
            expected_keywords = EXPECTED_ADDRESS_KEYWORDS.get(country, [])
            if any(kw.lower() in current_addr.lower() for kw in expected_keywords) or zipcode in current_addr:
                print(f"[DEBUG] 邮编已设置正确: {current_addr}", file=sys.stderr)
                return True, current_addr
        except:
            pass

        try:
            # 步骤1: 点击左上角定位图标
            location_btn = page.locator('#nav-global-location-popover-link').first
            await location_btn.click(timeout=5000)
            print(f"[DEBUG] 已点击定位图标", file=sys.stderr)

            # 步骤2: 等待弹窗完全出现
            await page.wait_for_timeout(2500)

            # 步骤3: 查找邮编输入框 (多种选择器尝试)
            zip_input = None
            zip_selectors = [
                '#GLUXZipUpdateInput',
                'input[id*="ZipUpdate"]',
                'input[data-action*="GLUXPostal"]',
                '.a-popover-modal input[type="text"]',
            ]

            for selector in zip_selectors:
                try:
                    input_elem = page.locator(selector).first
                    if await input_elem.is_visible(timeout=2000):
                        zip_input = input_elem
                        print(f"[DEBUG] 找到邮编输入框: {selector}", file=sys.stderr)
                        break
                except:
                    continue

            if zip_input is None:
                print(f"[DEBUG] 未找到邮编输入框，尝试点击其他链接", file=sys.stderr)
                # 可能需要先点击 "更改邮编" 链接
                change_links = [
                    '#GLUXChangePostalCodeLink',
                    'a[id*="ChangePostalCode"]',
                    'text=Postleitzahl',  # 德语
                    'text=code postal',   # 法语
                    'text=postal code',   # 英语
                ]
                for link_selector in change_links:
                    try:
                        link = page.locator(link_selector).first
                        if await link.is_visible(timeout=1000):
                            await link.click()
                            await page.wait_for_timeout(1500)
                            # 再次查找输入框
                            zip_input = page.locator('#GLUXZipUpdateInput').first
                            if await zip_input.is_visible(timeout=2000):
                                print(f"[DEBUG] 点击链接后找到邮编输入框", file=sys.stderr)
                                break
                    except:
                        continue

            if zip_input is None:
                print(f"[DEBUG] 仍未找到邮编输入框", file=sys.stderr)
                # 关闭弹窗重试
                await page.keyboard.press('Escape')
                await page.wait_for_timeout(1000)
                continue

            # 步骤4: 清空并输入邮编 (模拟人工输入)
            await zip_input.click()
            await zip_input.fill('')
            await page.wait_for_timeout(300)
            await zip_input.type(zipcode, delay=50)
            print(f"[DEBUG] 已输入邮编: {zipcode}", file=sys.stderr)
            await page.wait_for_timeout(500)

            # 步骤5: 点击应用按钮
            apply_btn = None
            apply_selectors = [
                '#GLUXZipUpdate',
                'input[id*="GLUXZipUpdate"]',
                'span[id*="GLUXZipUpdate"] input',
                '.a-button-input[aria-labelledby*="ZipUpdate"]',
            ]

            for selector in apply_selectors:
                try:
                    btn = page.locator(selector).first
                    if await btn.is_visible(timeout=1000):
                        apply_btn = btn
                        break
                except:
                    continue

            if apply_btn:
                await apply_btn.click()
                print(f"[DEBUG] 已点击应用按钮", file=sys.stderr)
            else:
                # 尝试按回车键
                await zip_input.press('Enter')
                print(f"[DEBUG] 按回车键提交", file=sys.stderr)

            await page.wait_for_timeout(3000)

            # 步骤6: 如果有确认/完成按钮，点击它
            done_selectors = [
                '#GLUXConfirmClose',
                'button[name="glowDoneButton"]',
                '.a-popover-footer button',
                'input[data-action*="Confirm"]',
            ]

            for selector in done_selectors:
                try:
                    done_btn = page.locator(selector).first
                    if await done_btn.is_visible(timeout=2000):
                        await done_btn.click()
                        print(f"[DEBUG] 点击确认按钮", file=sys.stderr)
                        await page.wait_for_timeout(2000)
                        break
                except:
                    continue

            # 确保弹窗已关闭
            await page.keyboard.press('Escape')
            await page.wait_for_timeout(1500)

            # 步骤7: 验证地址是否设置成功
            await page.wait_for_timeout(1000)
            try:
                address_text = await page.locator('#glow-ingress-line2').text_content(timeout=3000)
                address_text = address_text.strip() if address_text else ""
                print(f"[DEBUG] 当前配送地址显示: {address_text}", file=sys.stderr)

                # 检查地址是否包含目标国家的关键词
                expected_keywords = EXPECTED_ADDRESS_KEYWORDS.get(country, [])
                if any(kw.lower() in address_text.lower() for kw in expected_keywords):
                    print(f"[DEBUG] 邮编设置成功!", file=sys.stderr)
                    return True, address_text
                elif zipcode in address_text:
                    print(f"[DEBUG] 邮编设置成功 (邮编可见)!", file=sys.stderr)
                    return True, address_text
                else:
                    print(f"[DEBUG] 地址验证失败，当前显示: {address_text}", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 读取地址失败: {e}", file=sys.stderr)

            # 关闭可能残留的弹窗
            await page.keyboard.press('Escape')
            await page.wait_for_timeout(500)

        except Exception as e:
            print(f"[DEBUG] 设置邮编出错: {e}", file=sys.stderr)
            await page.keyboard.press('Escape')
            await page.wait_for_timeout(1000)

    # 所有尝试都失败
    try:
        address_text = await page.locator('#glow-ingress-line2').text_content(timeout=2000)
        return False, address_text.strip() if address_text else ""
    except:
        return False, ""


async def search_keyword(keyword: str, target_asin: str, country: str, max_pages: int = 5, proxy: str = None, headless = "new") -> dict:
    """
    使用 Playwright 搜索关键词并返回目标ASIN的排名

    参数:
        keyword: 搜索关键词
        target_asin: 目标商品ASIN
        country: 国家代码 (DE, FR, UK, US, IT, ES)
        max_pages: 最大搜索页数
        proxy: 代理地址，格式如 "http://127.0.0.1:7897"
        headless: 无头模式 ("new"=新版无头模式, True=传统无头, False=显示浏览器)
    """
    config = COUNTRY_CONFIG.get(country, COUNTRY_CONFIG["US"])

    result = {
        "keyword": keyword,
        "target_asin": target_asin,
        "country": country,
        "organic_rank": None,
        "organic_page": None,
        "sponsored_rank": None,
        "sponsored_page": None,
        "sponsored_type": None,
        "product_info": None,
        "organic_top_50": [],
        "sponsored_top_20": [],
        "checked_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "error": None,
        "warning": None,
        "delivery_address": None  # 记录实际设置的配送地址
    }

    organic_position = 0
    sponsored_position = 0
    found_organic = False
    found_sponsored = False

    async with async_playwright() as p:
        # 配置浏览器启动选项
        launch_options = {}

        # 处理 headless 模式
        if headless == "new":
            # 新版无头模式：更接近真实浏览器行为
            launch_options["headless"] = True
            launch_options["args"] = ["--headless=new"]
        elif headless == False:
            # 有头模式：正常显示浏览器
            launch_options["headless"] = False
        else:
            launch_options["headless"] = headless

        # 配置代理
        if proxy:
            launch_options["proxy"] = {"server": proxy}
            print(f"[DEBUG] 使用代理: {proxy}", file=sys.stderr)

        browser = await p.chromium.launch(**launch_options)

        # 国家对应的地理位置坐标
        geo_locations = {
            "DE": {"latitude": 52.5200, "longitude": 13.4050},  # 柏林
            "FR": {"latitude": 48.8566, "longitude": 2.3522},   # 巴黎
            "UK": {"latitude": 51.5074, "longitude": -0.1278},  # 伦敦
            "US": {"latitude": 40.7128, "longitude": -74.0060}, # 纽约
            "IT": {"latitude": 41.9028, "longitude": 12.4964},  # 罗马
            "ES": {"latitude": 40.4168, "longitude": -3.7038},  # 马德里
        }

        # 创建上下文，设置语言、地区和地理位置
        context = await browser.new_context(
            locale=config['language'],
            timezone_id="Europe/Berlin" if country == "DE" else "Europe/Paris" if country == "FR" else "Europe/London" if country == "UK" else "Europe/Rome" if country == "IT" else "Europe/Madrid" if country == "ES" else "America/New_York",
            viewport={"width": 1920, "height": 1080},
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            geolocation=geo_locations.get(country, geo_locations["DE"]),
            permissions=["geolocation"]
        )

        page = await context.new_page()

        # 设置额外的请求头
        await page.set_extra_http_headers({
            'Accept-Language': f'{config["language"]},en;q=0.9',
        })

        # 设置 Amazon 地区相关的 Cookie
        country_cookies = {
            'DE': [
                {'name': 'lc-acbde', 'value': 'de_DE', 'domain': '.amazon.de', 'path': '/'},
                {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.de', 'path': '/'},
            ],
            'FR': [
                {'name': 'lc-acbfr', 'value': 'fr_FR', 'domain': '.amazon.fr', 'path': '/'},
                {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.fr', 'path': '/'},
            ],
            'UK': [
                {'name': 'lc-acbuk', 'value': 'en_GB', 'domain': '.amazon.co.uk', 'path': '/'},
                {'name': 'i18n-prefs', 'value': 'GBP', 'domain': '.amazon.co.uk', 'path': '/'},
            ],
            'IT': [
                {'name': 'lc-acbit', 'value': 'it_IT', 'domain': '.amazon.it', 'path': '/'},
                {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.it', 'path': '/'},
            ],
            'ES': [
                {'name': 'lc-acbes', 'value': 'es_ES', 'domain': '.amazon.es', 'path': '/'},
                {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.es', 'path': '/'},
            ],
        }
        if country in country_cookies:
            await context.add_cookies(country_cookies[country])

        try:
            # 步骤1: 访问主页
            print(f"[DEBUG] 访问 {config['base_url']}...", file=sys.stderr)
            await page.goto(config['base_url'], wait_until="domcontentloaded", timeout=30000)

            # 检测并处理中间页面（继续购物按钮等）
            for retry in range(3):
                await page.wait_for_timeout(2000)

                # 检查是否有"继续购物"按钮（法语/英语/德语等）
                continue_buttons = [
                    'text=Continuer les achats',
                    'text=Continue shopping',
                    'text=Weiter einkaufen',
                    'text=Continua gli acquisti',
                    'text=Continuar comprando',
                    'input[type="submit"]',
                    'button:has-text("Continuer")',
                    'a:has-text("Continuer")',
                ]

                clicked = False
                for btn_selector in continue_buttons:
                    try:
                        btn = page.locator(btn_selector).first
                        if await btn.is_visible(timeout=2000):
                            await btn.click()
                            print(f"[DEBUG] 点击了继续按钮: {btn_selector}", file=sys.stderr)
                            clicked = True
                            await page.wait_for_timeout(3000)
                            break
                    except:
                        continue

                if not clicked:
                    break

            # 处理 Cookie 同意弹窗 (欧洲站点常见)
            try:
                cookie_btn = page.locator('#sp-cc-accept')
                if await cookie_btn.is_visible(timeout=3000):
                    await cookie_btn.click()
                    print(f"[DEBUG] 已接受 Cookie", file=sys.stderr)
                    await page.wait_for_timeout(1000)
            except:
                pass

            # 步骤2: 使用新的邮编设置函数（带重试和验证）
            print(f"[DEBUG] 设置配送地址: {config['zipcode']}...", file=sys.stderr)
            address_success, address_text = await set_delivery_address(page, country, config['zipcode'], max_retries=3)

            result['delivery_address'] = address_text
            if address_success:
                print(f"[DEBUG] 邮编设置成功: {address_text}", file=sys.stderr)
            else:
                print(f"[DEBUG] 邮编设置可能未完全成功: {address_text}", file=sys.stderr)
                result['warning'] = f'配送地址设置可能未生效，当前显示: {address_text}'

            # 等待页面稳定
            await page.wait_for_timeout(1000)

            # 步骤3: 执行搜索
            encoded_keyword = quote_plus(keyword)
            market_param = f"&{config['market_param']}" if config['market_param'] else ""

            for page_num in range(1, max_pages + 1):
                # 自然排名和广告排名都找到了才停止搜索
                if found_organic and found_sponsored:
                    print(f"[DEBUG] 自然排名和广告排名都已找到，停止搜索", file=sys.stderr)
                    break

                search_url = f"{config['base_url']}/s?k={encoded_keyword}{market_param}"
                if page_num > 1:
                    search_url += f"&page={page_num}"

                print(f"[DEBUG] 搜索第 {page_num} 页...", file=sys.stderr)
                await page.goto(search_url, wait_until="domcontentloaded", timeout=30000)
                await page.wait_for_timeout(5000)  # 等待广告完全加载

                # 滚动页面以触发懒加载广告
                await page.evaluate('window.scrollTo(0, 300)')
                await page.wait_for_timeout(500)
                await page.evaluate('window.scrollTo(0, 0)')
                await page.wait_for_timeout(500)

                # 检查是否有验证码
                try:
                    robot_check = page.locator('text=Robot Check')
                    if await robot_check.is_visible(timeout=1000):
                        result['error'] = "检测到验证码页面"
                        break
                except:
                    pass

                # 获取页面内容
                content = await page.content()

                # 检测顶部广告位置计数
                banner_product_count = 0  # 横幅中的产品数量
                has_video_ad = False      # 是否有视频广告

                # 1. 检测品牌横幅广告 (Sponsored Brands) 中的产品数量
                try:
                    # 多种横幅广告选择器（不同站点可能使用不同的类名）
                    banner_selectors = [
                        '[class*="sbx-desktop"]',
                        '[class*="_bXVsd_container"]',
                        '[class*="sb-desktop"]',
                        '[data-component-type="sbx"]',
                        '[data-component-type="sp-sponsored-brands"]',
                        '[class*="AdHolder"]',
                        '[class*="s-top-slot"] [class*="a-carousel"]',
                        # 更多通用选择器
                        '[data-component-type*="brand"]',
                        '.s-top-slot [class*="sponsored"]',
                        '[class*="sponsored-brand"]',
                        '.puis-carousel',
                        '[class*="brands-storefronts"]',
                        # 使用 cel_widget_id 属性
                        '[cel_widget_id*="MAIN-SEARCH_RESULTS-SBX"]',
                        '[cel_widget_id*="sponsoredBrands"]',
                    ]

                    sbx_container = None
                    for selector in banner_selectors:
                        try:
                            loc = page.locator(selector).first
                            count = await loc.count()
                            if count > 0:
                                sbx_container = loc
                                print(f"[DEBUG] 找到横幅广告容器，选择器: {selector}", file=sys.stderr)
                                break
                        except:
                            continue

                    if sbx_container is not None:
                        # 提取横幅中的所有 ASIN
                        sbx_html = await sbx_container.inner_html()

                        # 方法1: 从 /dp/ 链接中提取
                        banner_asins = re.findall(r'/dp/([A-Z0-9]{10})', sbx_html)

                        # 方法2: 从 lp_asins URL参数中提取 (法国站等使用这种格式)
                        if not banner_asins:
                            lp_asins_match = re.search(r'lp_asins=([A-Z0-9%,]+)', sbx_html)
                            if lp_asins_match:
                                from urllib.parse import unquote
                                lp_asins_str = unquote(lp_asins_match.group(1))
                                banner_asins = [a for a in lp_asins_str.split(',') if len(a) == 10 and a.isalnum()]
                                print(f"[DEBUG] 从 lp_asins 提取到 ASINs: {banner_asins}", file=sys.stderr)

                        # 方法3: 从 data-asin 属性提取
                        if not banner_asins:
                            banner_asins = re.findall(r'data-asin="([A-Z0-9]{10})"', sbx_html)

                        # 方法4: 从 asin= 参数提取
                        if not banner_asins:
                            banner_asins = re.findall(r'[?&]asin=([A-Z0-9]{10})', sbx_html)

                        banner_asins = list(dict.fromkeys(banner_asins))  # 去重保持顺序
                        banner_product_count = len(banner_asins)

                        if banner_product_count > 0:
                            print(f"[DEBUG] 横幅广告中有 {banner_product_count} 个产品: {banner_asins}", file=sys.stderr)

                            # 检查目标 ASIN 是否在横幅中
                            if not found_sponsored:
                                for idx, asin in enumerate(banner_asins, 1):
                                    if asin.upper() == target_asin.upper():
                                        result['sponsored_rank'] = idx
                                        result['sponsored_page'] = page_num
                                        result['sponsored_type'] = 'brand_banner'
                                        found_sponsored = True
                                        print(f"[DEBUG] 在横幅广告中找到目标ASIN: 广告第{idx}位", file=sys.stderr)
                                        break
                        else:
                            print(f"[DEBUG] 横幅广告容器存在，但未找到产品ASIN", file=sys.stderr)
                    else:
                        print(f"[DEBUG] 未检测到横幅广告容器", file=sys.stderr)
                except Exception as e:
                    print(f"[DEBUG] 横幅广告检测异常: {e}", file=sys.stderr)

                # 2. 检测视频广告 (SBV) - 视频广告在页面中间，需要计算实际位置
                video_ad_position = None
                video_ad_asin = None
                video_ad_dom_index = None  # 视频广告在DOM中的位置
                try:
                    # 视频广告选择器
                    sbv_selectors = [
                        '[class*="sbv-video"]',
                        '[class*="video-single-product"]',
                        '[data-component-type="sbv"]',
                        '[cel_widget_id*="VIDEO"]',
                    ]

                    for sbv_selector in sbv_selectors:
                        sbv_loc = page.locator(sbv_selector).first
                        if await sbv_loc.count() > 0:
                            has_video_ad = True
                            sbv_html = await sbv_loc.inner_html()

                            # 提取视频广告中的ASIN
                            video_asins = re.findall(r'/dp/([A-Z0-9]{10})', sbv_html)
                            if not video_asins:
                                # 尝试从 lp_asins 参数提取
                                lp_match = re.search(r'lp_asins=([A-Z0-9%,]+)', sbv_html)
                                if lp_match:
                                    from urllib.parse import unquote
                                    video_asins = [a for a in unquote(lp_match.group(1)).split(',') if len(a) == 10]

                            if video_asins:
                                video_ad_asin = video_asins[0]

                                # 获取视频广告在搜索结果中的DOM位置
                                # 通过JavaScript计算视频广告相对于搜索结果的位置
                                try:
                                    video_ad_dom_index = await page.evaluate(f'''() => {{
                                        const videoAd = document.querySelector('{sbv_selector}');
                                        const searchResults = document.querySelectorAll('[data-component-type="s-search-result"]');
                                        if (!videoAd || !searchResults.length) return null;

                                        const videoRect = videoAd.getBoundingClientRect();
                                        let count = 0;
                                        for (let i = 0; i < searchResults.length; i++) {{
                                            const resultRect = searchResults[i].getBoundingClientRect();
                                            // 如果搜索结果在视频广告之前（Y坐标更小）
                                            if (resultRect.top < videoRect.top) {{
                                                count++;
                                            }} else {{
                                                break;
                                            }}
                                        }}
                                        return count;
                                    }}''')
                                    print(f"[DEBUG] 检测到视频广告，ASIN: {video_ad_asin}，在第{video_ad_dom_index}个搜索结果之后", file=sys.stderr)
                                except:
                                    print(f"[DEBUG] 检测到视频广告，ASIN: {video_ad_asin}", file=sys.stderr)
                            break
                except Exception as e:
                    print(f"[DEBUG] 视频广告检测异常: {e}", file=sys.stderr)

                # 3. 检测顶部广告产品区块（Top Sponsored Products）- 在横幅下方，搜索结果上方的独立广告区
                top_sponsored_asins = []
                try:
                    # 多种顶部广告区选择器
                    top_ad_selectors = [
                        '[data-component-type="sp-sponsored-products"]',
                        '[cel_widget_id*="MAIN-TOP_BANNER"]',
                        '[cel_widget_id*="TOP_BANNER_SP"]',
                        '.s-top-slot [data-component-type*="sp-"]',
                        '[data-component-type="s-ads-metrics"]',
                    ]

                    for selector in top_ad_selectors:
                        try:
                            top_ad_container = page.locator(selector).first
                            if await top_ad_container.count() > 0:
                                top_ad_html = await top_ad_container.inner_html()
                                # 提取 ASIN
                                found_asins = re.findall(r'/dp/([A-Z0-9]{10})', top_ad_html)
                                if not found_asins:
                                    found_asins = re.findall(r'data-asin="([A-Z0-9]{10})"', top_ad_html)
                                if found_asins:
                                    top_sponsored_asins = list(dict.fromkeys(found_asins))
                                    print(f"[DEBUG] 顶部广告区找到 {len(top_sponsored_asins)} 个产品: {top_sponsored_asins[:5]}...", file=sys.stderr)

                                    # 检查目标 ASIN 是否在顶部广告区
                                    if not found_sponsored:
                                        for idx, asin in enumerate(top_sponsored_asins, 1):
                                            if asin.upper() == target_asin.upper():
                                                # 位置 = 横幅数 + 在顶部广告区的位置
                                                result['sponsored_rank'] = banner_product_count + idx
                                                result['sponsored_page'] = page_num
                                                result['sponsored_type'] = 'product_ad'
                                                found_sponsored = True
                                                print(f"[DEBUG] 在顶部广告区找到目标ASIN: 广告第{banner_product_count + idx}位", file=sys.stderr)
                                                break
                                    break
                        except:
                            continue
                except Exception as e:
                    print(f"[DEBUG] 顶部广告区检测异常: {e}", file=sys.stderr)

                # 普通广告起始位置 = 横幅产品数 + 顶部广告区产品数
                top_ad_count = banner_product_count + len(top_sponsored_asins)

                # 解析搜索结果 - 获取主搜索区域的所有子元素（包括视频广告）
                search_results = await page.locator('[data-component-type="s-search-result"]').all()
                print(f"[DEBUG] 第{page_num}页找到 {len(search_results)} 个搜索结果", file=sys.stderr)

                # 分开跟踪广告位和自然位（允许同一ASIN同时出现在两种位置）
                seen_organic_asins = set()
                seen_sponsored_asins = set()
                page_organic_position = 0   # 页内自然位置
                page_sponsored_position = top_ad_count  # 广告位置从顶部广告数量开始计数
                sponsored_before_video = 0  # 视频广告之前的普通广告数

                for idx, item in enumerate(search_results, 1):
                    try:
                        asin = await item.get_attribute('data-asin')

                        if not asin or len(asin) != 10:
                            continue

                        # 检查是否是广告 - 多种方式
                        item_html = await item.inner_html()
                        is_sponsored = bool(re.search(r'Sponsored|Sponsorisé|Gesponsert|Sponsorizzato|Patrocinado|Anzeige', item_html, re.IGNORECASE))

                        # 额外检测：通过 data-component-type 属性
                        if not is_sponsored:
                            comp_type = await item.get_attribute('data-component-type')
                            if comp_type and 'sp-' in comp_type.lower():
                                is_sponsored = True

                        # 额外检测：通过 AdHolder 类名
                        if not is_sponsored:
                            item_class = await item.get_attribute('class') or ''
                            if 'AdHolder' in item_class or 'sponsored' in item_class.lower():
                                is_sponsored = True

                        # 额外检测：通过 aria-label 包含 sponsored 或广告相关词
                        if not is_sponsored:
                            if 'sponsored' in item_html.lower() or 'anzeige' in item_html.lower():
                                is_sponsored = True

                        if is_sponsored:
                            # 广告位：独立计数
                            if asin not in seen_sponsored_asins:
                                seen_sponsored_asins.add(asin)
                                page_sponsored_position += 1

                                # 检查这个广告是否在视频广告之前
                                if video_ad_dom_index is not None and idx <= video_ad_dom_index:
                                    sponsored_before_video += 1
                                sponsored_position += 1

                                if len(result['sponsored_top_20']) < 20:
                                    result['sponsored_top_20'].append(asin)

                                if asin.upper() == target_asin.upper() and not found_sponsored:
                                    result['sponsored_rank'] = page_sponsored_position  # 页内广告位置
                                    result['sponsored_page'] = page_num
                                    result['sponsored_type'] = 'product_ad'
                                    found_sponsored = True
                                    print(f"[DEBUG] 找到目标ASIN广告排名: 第{page_num}页广告第{page_sponsored_position}位", file=sys.stderr)
                        else:
                            # 自然位：独立计数
                            if asin not in seen_organic_asins:
                                seen_organic_asins.add(asin)
                                page_organic_position += 1
                                organic_position += 1

                                if len(result['organic_top_50']) < 50:
                                    result['organic_top_50'].append(asin)

                                if asin.upper() == target_asin.upper() and not found_organic:
                                    result['organic_rank'] = page_organic_position  # 页内自然位置
                                    result['organic_page'] = page_num
                                    found_organic = True
                                    print(f"[DEBUG] 找到目标ASIN自然排名: 第{page_num}页自然第{page_organic_position}位", file=sys.stderr)

                                    # 提取产品信息
                                    try:
                                        title_elem = item.locator('h2')
                                        title = await title_elem.text_content() if await title_elem.count() > 0 else None

                                        price_elem = item.locator('.a-price .a-offscreen')
                                        price = await price_elem.first.text_content() if await price_elem.count() > 0 else None

                                        img_elem = item.locator('img.s-image')
                                        img_url = await img_elem.get_attribute('src') if await img_elem.count() > 0 else None

                                        result['product_info'] = {
                                            'asin': asin,
                                            'title': title.strip() if title else None,
                                            'price': price.strip() if price else None,
                                            'image_url': img_url
                                        }
                                    except:
                                        pass
                    except Exception as e:
                        continue

                # 处理视频广告：如果目标ASIN在视频广告中且还没找到广告排名
                if has_video_ad and video_ad_asin and not found_sponsored:
                    if video_ad_asin.upper() == target_asin.upper():
                        # 视频广告位置 = 横幅产品数 + 顶部广告区数 + 视频广告之前的普通广告数 + 1
                        video_position = banner_product_count + len(top_sponsored_asins) + sponsored_before_video + 1

                        result['sponsored_rank'] = video_position
                        result['sponsored_page'] = page_num
                        result['sponsored_type'] = 'video_ad'
                        found_sponsored = True
                        print(f"[DEBUG] 在视频广告中找到目标ASIN: 广告第{video_position}位 (横幅:{banner_product_count} + 顶部广告:{len(top_sponsored_asins)} + 搜索结果中广告:{sponsored_before_video} + 1)", file=sys.stderr)


                # 页面间延迟（未全部找到才继续）
                if page_num < max_pages and not (found_organic and found_sponsored):
                    await page.wait_for_timeout(2000)

            # 如果没有找到产品信息，尝试从详情页获取
            if result['product_info'] is None:
                try:
                    print(f"[DEBUG] 从详情页获取产品信息...", file=sys.stderr)
                    detail_url = f"{config['base_url']}/dp/{target_asin}"
                    await page.goto(detail_url, wait_until="domcontentloaded", timeout=30000)
                    await page.wait_for_timeout(2000)

                    title = None
                    price = None
                    img_url = None

                    try:
                        title_elem = page.locator('#productTitle')
                        if await title_elem.count() > 0:
                            title = await title_elem.text_content()
                    except:
                        pass

                    try:
                        price_elem = page.locator('.a-price .a-offscreen').first
                        if await page.locator('.a-price .a-offscreen').count() > 0:
                            price = await price_elem.text_content()
                    except:
                        pass

                    try:
                        img_elem = page.locator('#landingImage')
                        if await img_elem.count() > 0:
                            img_url = await img_elem.get_attribute('src')
                    except:
                        pass

                    result['product_info'] = {
                        'asin': target_asin,
                        'title': title.strip() if title else None,
                        'price': price.strip() if price else None,
                        'image_url': img_url
                    }
                except:
                    pass

        except Exception as e:
            result['error'] = str(e)
            print(f"[DEBUG] 错误: {e}", file=sys.stderr)

        finally:
            await browser.close()

    return result


async def search_keywords_batch(keywords_list: list, max_pages: int = 5, headless = "new") -> list:
    """
    批量搜索关键词 - 按国家分组，复用浏览器实例

    参数:
        keywords_list: [(id, keyword, asin, country), ...]
        max_pages: 最大搜索页数
        headless: 无头模式 ("new"=新版无头模式, True=传统无头, False=显示浏览器)

    返回: [(id, result), ...]
    """
    # 按国家分组
    country_groups = {}
    for item in keywords_list:
        monitoring_id, keyword, asin, country = item
        country = country.upper()
        if country not in country_groups:
            country_groups[country] = []
        country_groups[country].append((monitoring_id, keyword, asin))

    print(f"[DEBUG] 共 {len(keywords_list)} 个关键词，分为 {len(country_groups)} 个国家组", file=sys.stderr)

    all_results = []

    async with async_playwright() as p:
        for country, keywords in country_groups.items():
            print(f"[DEBUG] 开始处理 {country} 站 ({len(keywords)} 个关键词)...", file=sys.stderr)

            config = COUNTRY_CONFIG.get(country, COUNTRY_CONFIG["US"])

            # 为这个国家打开一个浏览器
            launch_options = {}
            if headless == "new":
                launch_options["headless"] = True
                launch_options["args"] = ["--headless=new"]
            elif headless == False:
                # 有头模式：正常显示浏览器
                launch_options["headless"] = False
            else:
                launch_options["headless"] = headless
            browser = await p.chromium.launch(**launch_options)

            # 国家对应的地理位置坐标
            geo_locations = {
                "DE": {"latitude": 52.5200, "longitude": 13.4050},
                "FR": {"latitude": 48.8566, "longitude": 2.3522},
                "UK": {"latitude": 51.5074, "longitude": -0.1278},
                "US": {"latitude": 40.7128, "longitude": -74.0060},
                "IT": {"latitude": 41.9028, "longitude": 12.4964},
                "ES": {"latitude": 40.4168, "longitude": -3.7038},
            }

            context = await browser.new_context(
                locale=config['language'],
                timezone_id="Europe/Berlin" if country == "DE" else "Europe/Paris" if country == "FR" else "Europe/London" if country == "UK" else "Europe/Rome" if country == "IT" else "Europe/Madrid" if country == "ES" else "America/New_York",
                viewport={"width": 1920, "height": 1080},
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                geolocation=geo_locations.get(country, geo_locations["DE"]),
                permissions=["geolocation"]
            )

            # 设置Cookie
            country_cookies = {
                'DE': [
                    {'name': 'lc-acbde', 'value': 'de_DE', 'domain': '.amazon.de', 'path': '/'},
                    {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.de', 'path': '/'},
                ],
                'FR': [
                    {'name': 'lc-acbfr', 'value': 'fr_FR', 'domain': '.amazon.fr', 'path': '/'},
                    {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.fr', 'path': '/'},
                ],
                'UK': [
                    {'name': 'lc-acbuk', 'value': 'en_GB', 'domain': '.amazon.co.uk', 'path': '/'},
                    {'name': 'i18n-prefs', 'value': 'GBP', 'domain': '.amazon.co.uk', 'path': '/'},
                ],
                'IT': [
                    {'name': 'lc-acbit', 'value': 'it_IT', 'domain': '.amazon.it', 'path': '/'},
                    {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.it', 'path': '/'},
                ],
                'ES': [
                    {'name': 'lc-acbes', 'value': 'es_ES', 'domain': '.amazon.es', 'path': '/'},
                    {'name': 'i18n-prefs', 'value': 'EUR', 'domain': '.amazon.es', 'path': '/'},
                ],
            }
            if country in country_cookies:
                await context.add_cookies(country_cookies[country])

            page = await context.new_page()

            await page.set_extra_http_headers({
                'Accept-Language': f'{config["language"]},en;q=0.9',
            })

            try:
                # 访问主页并设置邮编（只需设置一次）
                print(f"[DEBUG] {country}: 访问主页并设置邮编...", file=sys.stderr)
                await page.goto(config['base_url'], wait_until="domcontentloaded", timeout=30000)

                # 处理Cookie弹窗
                try:
                    cookie_btn = page.locator('#sp-cc-accept')
                    if await cookie_btn.is_visible(timeout=3000):
                        await cookie_btn.click()
                        await page.wait_for_timeout(1000)
                except:
                    pass

                # 设置邮编
                address_success, address_text = await set_delivery_address(page, country, config['zipcode'], max_retries=5)
                if address_success:
                    print(f"[DEBUG] {country}: 邮编设置成功: {address_text}", file=sys.stderr)
                else:
                    print(f"[DEBUG] {country}: 邮编设置可能未成功: {address_text}", file=sys.stderr)

                # 处理这个国家的所有关键词
                for idx, (monitoring_id, keyword, target_asin) in enumerate(keywords, 1):
                    print(f"[DEBUG] {country}: 检测 {idx}/{len(keywords)} - {keyword}", file=sys.stderr)

                    result = {
                        "keyword": keyword,
                        "target_asin": target_asin,
                        "country": country,
                        "organic_rank": None,
                        "organic_page": None,
                        "sponsored_rank": None,
                        "sponsored_page": None,
                        "sponsored_type": None,
                        "product_info": None,
                        "organic_top_50": [],
                        "sponsored_top_20": [],
                        "checked_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
                        "error": None,
                        "warning": None,
                        "delivery_address": address_text
                    }

                    try:
                        # 搜索关键词
                        encoded_keyword = quote_plus(keyword)
                        market_param = f"&{config['market_param']}" if config['market_param'] else ""

                        found_organic = False
                        found_sponsored = False
                        organic_position = 0
                        sponsored_position = 0

                        for page_num in range(1, max_pages + 1):
                            # 自然排名和广告排名都找到了才停止搜索
                            if found_organic and found_sponsored:
                                break

                            search_url = f"{config['base_url']}/s?k={encoded_keyword}{market_param}"
                            if page_num > 1:
                                search_url += f"&page={page_num}"

                            await page.goto(search_url, wait_until="domcontentloaded", timeout=30000)
                            await page.wait_for_timeout(4000)

                            # 滚动页面以触发懒加载广告
                            await page.evaluate('window.scrollTo(0, 300)')
                            await page.wait_for_timeout(500)
                            await page.evaluate('window.scrollTo(0, 0)')

                            # 检查验证码
                            try:
                                if await page.locator('text=Robot Check').is_visible(timeout=1000):
                                    result['error'] = "检测到验证码"
                                    break
                            except:
                                pass

                            content = await page.content()

                            # 检测顶部广告位置计数
                            banner_product_count = 0
                            has_video_ad = False

                            # 1. 检测品牌横幅广告中的产品
                            try:
                                banner_selectors = [
                                    '[class*="sbx-desktop"]',
                                    '[class*="_bXVsd_container"]',
                                    '[class*="sb-desktop"]',
                                    '[data-component-type="sbx"]',
                                    '[data-component-type="sp-sponsored-brands"]',
                                    '[class*="AdHolder"]',
                                    '[class*="s-top-slot"] [class*="a-carousel"]',
                                ]

                                sbx_container = None
                                for selector in banner_selectors:
                                    try:
                                        loc = page.locator(selector).first
                                        count = await loc.count()
                                        if count > 0:
                                            sbx_container = loc
                                            print(f"[DEBUG] 找到横幅广告容器，选择器: {selector}", file=sys.stderr)
                                            break
                                    except:
                                        continue

                                if sbx_container is not None:
                                    sbx_html = await sbx_container.inner_html()

                                    # 方法1: 从 /dp/ 链接中提取
                                    banner_asins = re.findall(r'/dp/([A-Z0-9]{10})', sbx_html)

                                    # 方法2: 从 lp_asins URL参数中提取
                                    if not banner_asins:
                                        lp_asins_match = re.search(r'lp_asins=([A-Z0-9%,]+)', sbx_html)
                                        if lp_asins_match:
                                            from urllib.parse import unquote
                                            lp_asins_str = unquote(lp_asins_match.group(1))
                                            banner_asins = [a for a in lp_asins_str.split(',') if len(a) == 10 and a.isalnum()]

                                    # 方法3: 从 data-asin 属性提取
                                    if not banner_asins:
                                        banner_asins = re.findall(r'data-asin="([A-Z0-9]{10})"', sbx_html)

                                    # 方法4: 从 asin= 参数提取
                                    if not banner_asins:
                                        banner_asins = re.findall(r'[?&]asin=([A-Z0-9]{10})', sbx_html)

                                    banner_asins = list(dict.fromkeys(banner_asins))
                                    banner_product_count = len(banner_asins)

                                    if banner_product_count > 0 and not found_sponsored:
                                        for idx, asin in enumerate(banner_asins, 1):
                                            if asin.upper() == target_asin.upper():
                                                result['sponsored_rank'] = idx
                                                result['sponsored_page'] = page_num
                                                result['sponsored_type'] = 'brand_banner'
                                                found_sponsored = True
                                                break
                            except:
                                pass

                            # 2. 检测视频广告（只检测，位置计算放到后面）
                            video_ad_asin = None
                            video_ad_dom_index = None
                            try:
                                sbv_selectors = [
                                    '[class*="sbv-video"]',
                                    '[class*="video-single-product"]',
                                    '[data-component-type="sbv"]',
                                    '[cel_widget_id*="VIDEO"]',
                                ]
                                for sbv_selector in sbv_selectors:
                                    sbv_loc = page.locator(sbv_selector).first
                                    if await sbv_loc.count() > 0:
                                        has_video_ad = True
                                        sbv_html = await sbv_loc.inner_html()
                                        video_asins = re.findall(r'/dp/([A-Z0-9]{10})', sbv_html)
                                        if not video_asins:
                                            lp_match = re.search(r'lp_asins=([A-Z0-9%,]+)', sbv_html)
                                            if lp_match:
                                                from urllib.parse import unquote
                                                video_asins = [a for a in unquote(lp_match.group(1)).split(',') if len(a) == 10]
                                        if video_asins:
                                            video_ad_asin = video_asins[0]
                                            # 获取视频广告在搜索结果中的DOM位置
                                            try:
                                                video_ad_dom_index = await page.evaluate(f'''() => {{
                                                    const videoAd = document.querySelector('{sbv_selector}');
                                                    const searchResults = document.querySelectorAll('[data-component-type="s-search-result"]');
                                                    if (!videoAd || !searchResults.length) return null;
                                                    const videoRect = videoAd.getBoundingClientRect();
                                                    let count = 0;
                                                    for (let i = 0; i < searchResults.length; i++) {{
                                                        const resultRect = searchResults[i].getBoundingClientRect();
                                                        if (resultRect.top < videoRect.top) {{ count++; }} else {{ break; }}
                                                    }}
                                                    return count;
                                                }}''')
                                            except:
                                                pass
                                        break
                            except:
                                pass

                            # 普通广告起始位置 = 横幅产品数
                            top_ad_count = banner_product_count

                            # 解析搜索结果
                            search_results = await page.locator('[data-component-type="s-search-result"]').all()

                            seen_organic_asins = set()
                            seen_sponsored_asins = set()
                            page_organic_position = 0
                            page_sponsored_position = top_ad_count
                            sponsored_before_video = 0

                            for item in search_results:
                                try:
                                    asin = await item.get_attribute('data-asin')
                                    if not asin or len(asin) != 10:
                                        continue

                                    item_html = await item.inner_html()

                                    # 检测广告 - 多种方式
                                    is_sponsored = bool(re.search(r'Sponsored|Sponsorisé|Gesponsert|Sponsorizzato|Patrocinado|Anzeige', item_html, re.IGNORECASE))

                                    # 额外检测：通过 data-component-type 属性
                                    if not is_sponsored:
                                        comp_type = await item.get_attribute('data-component-type')
                                        if comp_type and 'sp-' in comp_type.lower():
                                            is_sponsored = True

                                    # 额外检测：通过 AdHolder 类名
                                    if not is_sponsored:
                                        item_class = await item.get_attribute('class') or ''
                                        if 'AdHolder' in item_class or 'sponsored' in item_class.lower():
                                            is_sponsored = True

                                    if is_sponsored:
                                        if asin not in seen_sponsored_asins:
                                            seen_sponsored_asins.add(asin)
                                            page_sponsored_position += 1

                                            # 跟踪视频广告之前的普通广告数
                                            current_idx = len(seen_organic_asins) + len(seen_sponsored_asins)
                                            if video_ad_dom_index is not None and current_idx <= video_ad_dom_index:
                                                sponsored_before_video += 1

                                            if len(result['sponsored_top_20']) < 20:
                                                result['sponsored_top_20'].append(asin)

                                            if asin.upper() == target_asin.upper() and not found_sponsored:
                                                result['sponsored_rank'] = page_sponsored_position
                                                result['sponsored_page'] = page_num
                                                result['sponsored_type'] = 'product_ad'
                                                found_sponsored = True
                                    else:
                                        if asin not in seen_organic_asins:
                                            seen_organic_asins.add(asin)
                                            page_organic_position += 1

                                            if len(result['organic_top_50']) < 50:
                                                result['organic_top_50'].append(asin)

                                            if asin.upper() == target_asin.upper() and not found_organic:
                                                result['organic_rank'] = page_organic_position
                                                result['organic_page'] = page_num
                                                found_organic = True

                                                # 提取产品信息
                                                try:
                                                    title_elem = item.locator('h2')
                                                    title = await title_elem.text_content() if await title_elem.count() > 0 else None
                                                    price_elem = item.locator('.a-price .a-offscreen')
                                                    price = await price_elem.first.text_content() if await price_elem.count() > 0 else None
                                                    img_elem = item.locator('img.s-image')
                                                    img_url = await img_elem.get_attribute('src') if await img_elem.count() > 0 else None

                                                    result['product_info'] = {
                                                        'asin': asin,
                                                        'title': title.strip() if title else None,
                                                        'price': price.strip() if price else None,
                                                        'image_url': img_url
                                                    }
                                                except:
                                                    pass
                                except:
                                    continue

                            # 处理视频广告：如果目标ASIN在视频广告中且还没找到广告排名
                            if has_video_ad and video_ad_asin and not found_sponsored:
                                if video_ad_asin.upper() == target_asin.upper():
                                    # 视频广告位置 = 横幅产品数 + 视频广告之前的普通广告数 + 1
                                    video_position = banner_product_count + sponsored_before_video + 1

                                    result['sponsored_rank'] = video_position
                                    result['sponsored_page'] = page_num
                                    result['sponsored_type'] = 'video_ad'
                                    found_sponsored = True

                            # 页面间延迟（未全部找到才继续）
                            if page_num < max_pages and not (found_organic and found_sponsored):
                                await page.wait_for_timeout(1500)

                        # 如果没找到产品信息，从详情页获取
                        if result['product_info'] is None:
                            try:
                                await page.goto(f"{config['base_url']}/dp/{target_asin}", wait_until="domcontentloaded", timeout=30000)
                                await page.wait_for_timeout(2000)

                                title = price = img_url = None
                                try:
                                    title_elem = page.locator('#productTitle')
                                    if await title_elem.count() > 0:
                                        title = await title_elem.text_content()
                                except:
                                    pass
                                try:
                                    price_elem = page.locator('.a-price .a-offscreen').first
                                    if await page.locator('.a-price .a-offscreen').count() > 0:
                                        price = await price_elem.text_content()
                                except:
                                    pass
                                try:
                                    img_elem = page.locator('#landingImage')
                                    if await img_elem.count() > 0:
                                        img_url = await img_elem.get_attribute('src')
                                except:
                                    pass

                                result['product_info'] = {
                                    'asin': target_asin,
                                    'title': title.strip() if title else None,
                                    'price': price.strip() if price else None,
                                    'image_url': img_url
                                }
                            except:
                                pass

                    except Exception as e:
                        result['error'] = str(e)

                    all_results.append((monitoring_id, result))

                    # 输出进度（实时）
                    progress = {
                        "type": "progress",
                        "monitoring_id": monitoring_id,
                        "result": result
                    }
                    output = json.dumps(progress, ensure_ascii=False)
                    sys.stdout.buffer.write(output.encode('utf-8'))
                    sys.stdout.buffer.write(b'\n')
                    sys.stdout.flush()

                    # 关键词间短延迟（同一国家内）
                    if idx < len(keywords):
                        await page.wait_for_timeout(2000)

            except Exception as e:
                print(f"[DEBUG] {country} 处理出错: {e}", file=sys.stderr)

            finally:
                await browser.close()

            # 国家间延迟
            await asyncio.sleep(2)

    return all_results


def main():
    if len(sys.argv) < 2:
        print(json.dumps({
            "error": "用法: python amazon_crawler_playwright.py <keyword> <asin> <country> [max_pages] [proxy] [headless]\n或: python amazon_crawler_playwright.py --batch"
        }))
        sys.exit(1)

    # 批量模式：从stdin读取JSON
    if sys.argv[1] == '--batch':
        # headless 参数: "new"=新版无头, "true"=传统无头, "false"=显示浏览器
        headless_arg = sys.argv[2].lower() if len(sys.argv) > 2 else "new"
        if headless_arg == "new":
            headless = "new"
        elif headless_arg == "false":
            headless = False
        else:
            headless = True
        max_pages = int(sys.argv[3]) if len(sys.argv) > 3 else 5

        # 从stdin读取JSON
        input_data = sys.stdin.read()
        keywords_list = json.loads(input_data)

        # 执行批量检测
        results = asyncio.run(search_keywords_batch(keywords_list, max_pages, headless))

        # 输出完成标记
        complete = {"type": "complete", "total": len(results)}
        output = json.dumps(complete, ensure_ascii=False)
        sys.stdout.buffer.write(output.encode('utf-8'))
        sys.stdout.buffer.write(b'\n')
        sys.exit(0)

    # 单个关键词模式
    if len(sys.argv) < 4:
        print(json.dumps({
            "error": "用法: python amazon_crawler_playwright.py <keyword> <asin> <country> [max_pages] [proxy] [headless]"
        }))
        sys.exit(1)

    keyword = sys.argv[1]
    target_asin = sys.argv[2]
    country = sys.argv[3].upper()
    max_pages = int(sys.argv[4]) if len(sys.argv) > 4 else 5
    proxy = sys.argv[5] if len(sys.argv) > 5 and sys.argv[5] != 'none' else None
    # headless 参数: "new"=新版无头, "true"=传统无头, "false"=显示浏览器
    headless_arg = sys.argv[6].lower() if len(sys.argv) > 6 else "new"
    if headless_arg == "new":
        headless = "new"
    elif headless_arg == "false":
        headless = False
    else:
        headless = True

    result = asyncio.run(search_keyword(keyword, target_asin, country, max_pages, proxy, headless))

    output = json.dumps(result, ensure_ascii=False)
    sys.stdout.buffer.write(output.encode('utf-8'))
    sys.stdout.buffer.write(b'\n')


if __name__ == "__main__":
    main()
