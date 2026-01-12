#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Amazon 竞品 Listing 信息爬虫 (Playwright 版本)
爬取产品详情页的标题、五点、描述、价格、评分等信息
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
        "language": "en-US",
        "zipcode": "10001",
        "currency": "USD"
    },
    "UK": {
        "base_url": "https://www.amazon.co.uk",
        "language": "en-GB",
        "zipcode": "SW1A 1AA",
        "currency": "GBP"
    },
    "DE": {
        "base_url": "https://www.amazon.de",
        "language": "de-DE",
        "zipcode": "10115",
        "currency": "EUR"
    },
    "FR": {
        "base_url": "https://www.amazon.fr",
        "language": "fr-FR",
        "zipcode": "75001",
        "currency": "EUR"
    },
    "IT": {
        "base_url": "https://www.amazon.it",
        "language": "it-IT",
        "zipcode": "00100",
        "currency": "EUR"
    },
    "ES": {
        "base_url": "https://www.amazon.es",
        "language": "es-ES",
        "zipcode": "28001",
        "currency": "EUR"
    },
    "JP": {
        "base_url": "https://www.amazon.co.jp",
        "language": "ja-JP",
        "zipcode": "100-0001",
        "currency": "JPY"
    },
    "CA": {
        "base_url": "https://www.amazon.ca",
        "language": "en-CA",
        "zipcode": "M5V 2H1",
        "currency": "CAD"
    },
    "AU": {
        "base_url": "https://www.amazon.com.au",
        "language": "en-AU",
        "zipcode": "2000",
        "currency": "AUD"
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
    'JP': ['Japan', '日本', '100-0001', 'Tokyo', '東京'],
    'CA': ['Canada', 'M5V', 'Toronto'],
    'AU': ['Australia', '2000', 'Sydney'],
}


async def set_delivery_address(page, country: str, zipcode: str, max_retries: int = 3) -> tuple:
    """
    设置配送地址 - 与排名监控保持一致
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
            await page.wait_for_timeout(2500)

            # 步骤2: 查找邮编输入框
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
                # 可能需要先点击 "更改邮编" 链接
                change_links = [
                    '#GLUXChangePostalCodeLink',
                    'a[id*="ChangePostalCode"]',
                    'text=Postleitzahl',
                    'text=code postal',
                    'text=postal code',
                ]
                for link_selector in change_links:
                    try:
                        link = page.locator(link_selector).first
                        if await link.is_visible(timeout=1000):
                            await link.click()
                            await page.wait_for_timeout(1500)
                            zip_input = page.locator('#GLUXZipUpdateInput').first
                            if await zip_input.is_visible(timeout=2000):
                                break
                    except:
                        continue

            if zip_input is None:
                await page.keyboard.press('Escape')
                await page.wait_for_timeout(1000)
                continue

            # 步骤3: 输入邮编
            # 日本站特殊处理：有两个邮编输入框（前3位和后4位）
            if country == 'JP':
                # 日本邮编格式: xxx-xxxx 或 xxxxxxx
                zip_parts = zipcode.replace('-', '')
                if len(zip_parts) == 7:
                    zip_prefix = zip_parts[:3]  # 前3位
                    zip_suffix = zip_parts[3:]  # 后4位
                else:
                    zip_prefix = zipcode
                    zip_suffix = ''

                # 查找日本站的两个邮编输入框
                jp_zip_selectors = [
                    ('#GLUXZipUpdateInput_0', '#GLUXZipUpdateInput_1'),
                    ('input[id*="ZipUpdateInput_0"]', 'input[id*="ZipUpdateInput_1"]'),
                    ('input[name="zipCode-0"]', 'input[name="zipCode-1"]'),
                ]

                jp_inputs_found = False
                for prefix_sel, suffix_sel in jp_zip_selectors:
                    try:
                        prefix_input = page.locator(prefix_sel).first
                        suffix_input = page.locator(suffix_sel).first
                        if await prefix_input.is_visible(timeout=1000) and await suffix_input.is_visible(timeout=1000):
                            # 输入前3位
                            await prefix_input.click()
                            await prefix_input.fill('')
                            await prefix_input.type(zip_prefix, delay=50)
                            print(f"[DEBUG] 已输入邮编前3位: {zip_prefix}", file=sys.stderr)

                            # 输入后4位
                            await suffix_input.click()
                            await suffix_input.fill('')
                            await suffix_input.type(zip_suffix, delay=50)
                            print(f"[DEBUG] 已输入邮编后4位: {zip_suffix}", file=sys.stderr)

                            jp_inputs_found = True
                            break
                    except:
                        continue

                # 如果没找到双输入框，尝试单输入框
                if not jp_inputs_found:
                    await zip_input.click()
                    await zip_input.fill('')
                    await page.wait_for_timeout(300)
                    await zip_input.type(zipcode, delay=50)
                    print(f"[DEBUG] 已输入邮编(单框): {zipcode}", file=sys.stderr)
            else:
                # 其他国家：单个输入框
                await zip_input.click()
                await zip_input.fill('')
                await page.wait_for_timeout(300)
                await zip_input.type(zipcode, delay=50)
                print(f"[DEBUG] 已输入邮编: {zipcode}", file=sys.stderr)

            await page.wait_for_timeout(500)

            # 步骤4: 点击应用按钮
            apply_selectors = [
                '#GLUXZipUpdate',
                'input[id*="GLUXZipUpdate"]',
                'span[id*="GLUXZipUpdate"] input',
            ]
            for selector in apply_selectors:
                try:
                    btn = page.locator(selector).first
                    if await btn.is_visible(timeout=1000):
                        await btn.click()
                        print(f"[DEBUG] 已点击应用按钮", file=sys.stderr)
                        break
                except:
                    continue
            else:
                await zip_input.press('Enter')

            await page.wait_for_timeout(3000)

            # 步骤5: 点击确认按钮
            done_selectors = [
                '#GLUXConfirmClose',
                'button[name="glowDoneButton"]',
                '.a-popover-footer button',
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

            await page.keyboard.press('Escape')
            await page.wait_for_timeout(1500)

            # 步骤6: 验证地址是否设置成功
            try:
                address_text = await page.locator('#glow-ingress-line2').text_content(timeout=3000)
                address_text = address_text.strip() if address_text else ""
                print(f"[DEBUG] 当前配送地址显示: {address_text}", file=sys.stderr)

                expected_keywords = EXPECTED_ADDRESS_KEYWORDS.get(country, [])
                if any(kw.lower() in address_text.lower() for kw in expected_keywords) or zipcode in address_text:
                    print(f"[DEBUG] 邮编设置成功!", file=sys.stderr)
                    return True, address_text
            except Exception as e:
                print(f"[DEBUG] 读取地址失败: {e}", file=sys.stderr)

            await page.keyboard.press('Escape')
            await page.wait_for_timeout(500)

        except Exception as e:
            print(f"[DEBUG] 设置邮编出错: {e}", file=sys.stderr)
            await page.keyboard.press('Escape')
            await page.wait_for_timeout(1000)

    # 所有重试都失败
    try:
        address_text = await page.locator('#glow-ingress-line2').text_content(timeout=2000)
        return False, address_text.strip() if address_text else ""
    except:
        return False, ""


# 国家对应的本地货币
COUNTRY_CURRENCIES = {
    'DE': 'EUR', 'FR': 'EUR', 'IT': 'EUR', 'ES': 'EUR',
    'UK': 'GBP', 'US': 'USD', 'JP': 'JPY', 'CA': 'CAD', 'AU': 'AUD',
}


async def set_currency(page, country: str) -> bool:
    """
    设置显示货币为本地货币（主要用于德国站等可能默认显示美元的情况）
    返回: 是否成功
    """
    target_currency = COUNTRY_CURRENCIES.get(country, 'EUR')
    currency_symbol = {'EUR': '€', 'GBP': '£', 'USD': '$', 'JPY': '¥', 'CAD': 'CA$', 'AUD': 'A$'}.get(target_currency, '€')

    try:
        # 检查当前页面价格是否已经是目标货币
        price_elem = page.locator('.a-price .a-offscreen').first
        if await price_elem.count() > 0:
            price_text = await price_elem.text_content() or ''
            # 如果价格已经是目标货币符号，不需要设置
            if currency_symbol in price_text:
                print(f"[DEBUG] 货币已经是 {target_currency}，无需设置", file=sys.stderr)
                return True
            # 检测到美元但目标是欧元，需要设置
            if '$' in price_text or 'USD' in price_text:
                print(f"[DEBUG] 检测到美元价格，需要切换到 {target_currency}", file=sys.stderr)

        print(f"[DEBUG] 尝试设置货币为 {target_currency}", file=sys.stderr)

        # 点击导航栏的国旗/语言图标（在搜索框后面）
        flag_selectors = [
            '#icp-nav-flyout',  # 国旗下拉框
            '#nav-icp-link-language-select',
            'a.nav-link[href*="language"]',
            '#nav-tools .icp-nav-link-inner',
        ]

        clicked = False
        for selector in flag_selectors:
            try:
                elem = page.locator(selector).first
                if await elem.count() > 0 and await elem.is_visible(timeout=2000):
                    await elem.click()
                    await page.wait_for_timeout(1500)
                    clicked = True
                    print(f"[DEBUG] 已点击语言/货币图标: {selector}", file=sys.stderr)
                    break
            except:
                continue

        if not clicked:
            print(f"[DEBUG] 未找到语言/货币图标", file=sys.stderr)
            return False

        # 等待设置页面加载
        await page.wait_for_timeout(1000)

        # 查找货币下拉框并选择 EUR
        # 德国站的货币下拉框在 "Währungseinstellungen" 部分
        currency_selected = False

        # 方法1: 查找 select 下拉框
        dropdown_selectors = [
            '#icp-sc-dropdown',
            'select[name*="currency"]',
            'select[id*="currency"]',
            '.a-native-dropdown[name*="currency"]',
            'select.a-native-dropdown',
        ]

        for selector in dropdown_selectors:
            try:
                dropdown = page.locator(selector).first
                if await dropdown.count() > 0:
                    # 尝试直接选择 EUR
                    try:
                        await dropdown.select_option(value=target_currency)
                        currency_selected = True
                        print(f"[DEBUG] 从下拉框选择货币: {target_currency}", file=sys.stderr)
                        break
                    except:
                        # 如果 value 不匹配，尝试用 label
                        try:
                            await dropdown.select_option(label=f"€ - EUR")
                            currency_selected = True
                            print(f"[DEBUG] 从下拉框选择货币 (label): EUR", file=sys.stderr)
                            break
                        except:
                            pass
            except:
                continue

        # 方法2: 如果没有找到 select，可能是自定义下拉框，需要点击展开
        if not currency_selected:
            # 查找当前显示 USD 的元素并点击
            usd_display_selectors = [
                'span:has-text("USD")',
                'div:has-text("$ - USD")',
                '.a-dropdown-prompt:has-text("USD")',
                '[class*="dropdown"]:has-text("USD")',
            ]

            for selector in usd_display_selectors:
                try:
                    elem = page.locator(selector).first
                    if await elem.count() > 0 and await elem.is_visible(timeout=1000):
                        await elem.click()
                        await page.wait_for_timeout(800)
                        print(f"[DEBUG] 已点击货币下拉框: {selector}", file=sys.stderr)

                        # 在展开的列表中选择 EUR
                        eur_options = [
                            'a:has-text("EUR")',
                            'li:has-text("EUR")',
                            'div:has-text("€ - EUR")',
                            '[data-value="EUR"]',
                        ]
                        for eur_sel in eur_options:
                            try:
                                eur_elem = page.locator(eur_sel).first
                                if await eur_elem.count() > 0 and await eur_elem.is_visible(timeout=500):
                                    await eur_elem.click()
                                    currency_selected = True
                                    print(f"[DEBUG] 已选择 EUR: {eur_sel}", file=sys.stderr)
                                    break
                            except:
                                continue
                        if currency_selected:
                            break
                except:
                    continue

        # 点击保存按钮 - 使用文本匹配（最可靠）
        save_btn_texts = [
            "Änderungen speichern",  # 德语 - 保存更改
            "Save Changes",  # 英语
            "Enregistrer les modifications",  # 法语
            "Salva modifiche",  # 意大利语
            "Guardar cambios",  # 西班牙语
        ]

        for text in save_btn_texts:
            try:
                # 方法1: 使用 get_by_role 查找按钮
                btn = page.get_by_role("button", name=text)
                if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                    await btn.first.click()
                    print(f"[DEBUG] 已点击保存按钮 (role): {text}", file=sys.stderr)
                    await page.wait_for_timeout(3000)  # 等待页面跳转
                    return True
            except:
                pass

            try:
                # 方法2: 使用 get_by_text 查找
                btn = page.get_by_text(text, exact=True)
                if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                    await btn.first.click()
                    print(f"[DEBUG] 已点击保存按钮 (text): {text}", file=sys.stderr)
                    await page.wait_for_timeout(3000)
                    return True
            except:
                pass

        # 方法3: 使用 CSS 选择器
        save_btn_selectors = [
            'input[type="submit"][value*="speichern"]',
            'input[type="submit"][value*="Save"]',
            'button:has-text("speichern")',
            'button:has-text("Save")',
            '.a-button-primary input[type="submit"]',
            '.a-button-primary button',
            '#icp-save-button',
            'input.a-button-input[type="submit"]',
        ]

        for selector in save_btn_selectors:
            try:
                btn = page.locator(selector).first
                if await btn.count() > 0 and await btn.is_visible(timeout=1000):
                    await btn.click()
                    print(f"[DEBUG] 已点击保存按钮 (css): {selector}", file=sys.stderr)
                    await page.wait_for_timeout(3000)
                    return True
            except:
                continue

        print(f"[DEBUG] 未找到保存按钮", file=sys.stderr)
        # 如果没有找到保存按钮，按 Escape 关闭
        await page.keyboard.press('Escape')
        await page.wait_for_timeout(500)
        return currency_selected

    except Exception as e:
        print(f"[DEBUG] 设置货币失败: {e}", file=sys.stderr)
        await page.keyboard.press('Escape')
        return False


async def fetch_listing_info(asin: str, country: str, headless="new") -> dict:
    """
    爬取单个 ASIN 的 Listing 信息

    返回:
    {
        "asin": str,
        "country": str,
        "title": str,
        "price": str,
        "rating": str,
        "review_count": int,
        "bsr_rank": str,
        "bullets": [str, ...],  # 五点描述列表
        "description": str,
        "fetched_at": str,
        "error": str | null
    }
    """
    config = COUNTRY_CONFIG.get(country.upper(), COUNTRY_CONFIG["US"])

    result = {
        "asin": asin,
        "country": country,
        "title": None,
        "price": None,
        "rating": None,
        "review_count": None,
        "bsr_rank": None,
        "date_first_available": None,
        "image_url": None,
        "bullets": [],
        "description": None,
        "fetched_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "error": None
    }

    async with async_playwright() as p:
        # 配置浏览器启动选项（与排名监控保持一致）
        launch_options = {}
        if headless == "new":
            launch_options["headless"] = True
            launch_options["args"] = ["--headless=new"]
        elif headless == False:
            launch_options["headless"] = False
        else:
            launch_options["headless"] = headless

        browser = await p.chromium.launch(**launch_options)

        # 国家对应的地理位置坐标（与排名监控一致）
        geo_locations = {
            "DE": {"latitude": 52.5200, "longitude": 13.4050},
            "FR": {"latitude": 48.8566, "longitude": 2.3522},
            "UK": {"latitude": 51.5074, "longitude": -0.1278},
            "US": {"latitude": 40.7128, "longitude": -74.0060},
            "IT": {"latitude": 41.9028, "longitude": 12.4964},
            "ES": {"latitude": 40.4168, "longitude": -3.7038},
            "JP": {"latitude": 35.6762, "longitude": 139.6503},
            "CA": {"latitude": 43.6532, "longitude": -79.3832},
            "AU": {"latitude": -33.8688, "longitude": 151.2093},
        }

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
            geolocation=geo_locations.get(country.upper(), geo_locations["US"]),
            permissions=["geolocation"]
        )

        page = await context.new_page()

        # 设置额外的请求头（与排名监控一致）
        await page.set_extra_http_headers({
            'Accept-Language': f'{config["language"]},en;q=0.9',
        })

        try:
            # 访问产品详情页
            detail_url = f"{config['base_url']}/dp/{asin}"
            print(f"[DEBUG] 访问: {detail_url}", file=sys.stderr)
            await page.goto(detail_url, wait_until="domcontentloaded", timeout=30000)
            await page.wait_for_timeout(3000)

            # 处理 Cookie 弹窗
            try:
                cookie_btn = page.locator('#sp-cc-accept')
                if await cookie_btn.is_visible(timeout=2000):
                    await cookie_btn.click()
                    await page.wait_for_timeout(1000)
            except:
                pass

            # 处理"继续购物"黄色按钮（地区确认页面）
            try:
                # 方法1: 使用 Playwright 的 text 定位器（最可靠）
                continue_texts = [
                    "Continuer les achats",  # 法语
                    "Continue shopping",      # 英语
                    "Weiter einkaufen",       # 德语
                    "Weiter shoppen",         # 德语变体
                    "Continua lo shopping",   # 意大利语
                    "Continua con gli acquisti",  # 意大利语变体
                    "Seguir comprando",       # 西班牙语
                    "ショッピングを続ける",     # 日语
                ]
                clicked = False
                for text in continue_texts:
                    try:
                        # 使用 get_by_role 查找按钮
                        btn = page.get_by_role("button", name=text)
                        if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                            print(f"[DEBUG] 发现继续购物按钮 (role): {text}", file=sys.stderr)
                            await btn.first.click()
                            await page.wait_for_timeout(2000)
                            clicked = True
                            break
                    except:
                        pass

                    if not clicked:
                        try:
                            # 使用 get_by_text 查找
                            btn = page.get_by_text(text, exact=True)
                            if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                                print(f"[DEBUG] 发现继续购物按钮 (text): {text}", file=sys.stderr)
                                await btn.first.click()
                                await page.wait_for_timeout(2000)
                                clicked = True
                                break
                        except:
                            pass

                # 方法2: 使用 CSS 选择器作为后备
                if not clicked:
                    css_selectors = [
                        'input[type="submit"][value*="Continuer"]',
                        'input[type="submit"][value*="Continue"]',
                        'span.a-button-text',  # 找到后检查文本
                        '.a-button-primary input[type="submit"]',
                    ]
                    for selector in css_selectors:
                        try:
                            elements = page.locator(selector)
                            count = await elements.count()
                            for i in range(count):
                                elem = elements.nth(i)
                                if await elem.is_visible(timeout=500):
                                    # 检查是否包含关键词
                                    text = await elem.text_content() or await elem.get_attribute('value') or ''
                                    if any(kw in text for kw in ['Continuer', 'Continue', 'Weiter', 'Continua', 'Seguir']):
                                        print(f"[DEBUG] 发现继续购物按钮 (css): {text}", file=sys.stderr)
                                        await elem.click()
                                        await page.wait_for_timeout(2000)
                                        clicked = True
                                        break
                            if clicked:
                                break
                        except:
                            pass
            except Exception as e:
                print(f"[DEBUG] 处理继续购物按钮出错: {e}", file=sys.stderr)

            # 设置配送地址/邮编（与排名监控一致）
            try:
                address_success, address_text = await set_delivery_address(page, country, config['zipcode'], max_retries=3)
                if address_success:
                    print(f"[DEBUG] 配送地址设置成功: {address_text}", file=sys.stderr)
                    # 刷新页面以获取正确的价格
                    await page.reload(wait_until="domcontentloaded", timeout=30000)
                    await page.wait_for_timeout(3000)

                    # 刷新后可能再次出现"继续购物"按钮，需要再次处理
                    try:
                        refresh_continue_texts = [
                            "Continuer les achats", "Continue shopping",
                            "Weiter einkaufen", "Weiter shoppen",
                            "Continua lo shopping", "Continua con gli acquisti",
                            "Seguir comprando"
                        ]
                        for text in refresh_continue_texts:
                            try:
                                btn = page.get_by_text(text, exact=True)
                                if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                                    print(f"[DEBUG] 刷新后再次发现继续购物按钮: {text}，点击", file=sys.stderr)
                                    await btn.first.click()
                                    await page.wait_for_timeout(2000)
                                    break
                            except:
                                continue
                    except:
                        pass
                else:
                    print(f"[DEBUG] 配送地址设置失败，使用当前地址: {address_text}", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 设置配送地址出错: {e}", file=sys.stderr)

            # 设置货币（处理德国站等可能显示美元的情况）
            try:
                await set_currency(page, country)
            except Exception as e:
                print(f"[DEBUG] 设置货币出错: {e}", file=sys.stderr)

            # 检查是否是有效的产品页
            try:
                not_found = page.locator('text=Page Not Found, text=Looking for something?')
                if await not_found.is_visible(timeout=1000):
                    result['error'] = "产品页面不存在"
                    return result
            except:
                pass

            # 1. 提取标题 (使用 span#productTitle 避免匹配到 hidden input)
            try:
                title_elem = page.locator('span#productTitle').first
                title = await title_elem.text_content(timeout=5000)
                result['title'] = title.strip() if title else None
                print(f"[DEBUG] 标题: {result['title'][:50]}..." if result['title'] else "[DEBUG] 未找到标题", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 提取标题失败: {e}", file=sys.stderr)

            # 1.5 提取主图 (与排名监控一致)
            try:
                img_selectors = [
                    '#landingImage',
                    '#imgBlkFront',
                    '#main-image',
                    '.a-dynamic-image',
                ]
                for img_sel in img_selectors:
                    img_elem = page.locator(img_sel).first
                    if await img_elem.count() > 0:
                        img_url = await img_elem.get_attribute('src')
                        if img_url and img_url.startswith('http'):
                            result['image_url'] = img_url
                            print(f"[DEBUG] 图片: {img_url[:60]}...", file=sys.stderr)
                            break
            except Exception as e:
                print(f"[DEBUG] 提取图片失败: {e}", file=sys.stderr)

            # 2. 提取价格 (与排名监控爬虫保持一致)
            try:
                price_elem = page.locator('.a-price .a-offscreen').first
                if await page.locator('.a-price .a-offscreen').count() > 0:
                    price = await price_elem.text_content()
                    if price:
                        result['price'] = price.strip()
                        print(f"[DEBUG] 价格: {result['price']}", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 提取价格失败: {e}", file=sys.stderr)

            # 3. 提取评分 (使用更精确的选择器，避免获取到变体或其他产品的评分)
            try:
                rating_found = False
                # 方法1: 从主评分区域获取 (最可靠)
                rating_selectors = [
                    '#acrPopover .a-icon-alt',  # 主评分弹出框
                    '#averageCustomerReviews .a-icon-alt',  # 平均评分区域
                    '#acrPopover span.a-icon-alt',
                    '#cm_cr_dp_d_rating_histogram .a-icon-alt',  # 评分直方图
                ]
                for selector in rating_selectors:
                    try:
                        rating_elem = page.locator(selector).first
                        if await rating_elem.count() > 0:
                            rating_text = await rating_elem.text_content() or ''
                            rating_match = re.search(r'([\d,\.]+)', rating_text)
                            if rating_match:
                                result['rating'] = rating_match.group(1).replace(',', '.')
                                print(f"[DEBUG] 评分: {result['rating']} (from {selector})", file=sys.stderr)
                                rating_found = True
                                break
                    except:
                        continue

                # 方法2: 如果上面没找到，使用更宽泛的选择器但限定在评论区域
                if not rating_found:
                    rating_elem = page.locator('#reviewsMedley .a-icon-star .a-icon-alt, #averageCustomerReviews_feature_div .a-icon-alt').first
                    if await rating_elem.count() > 0:
                        rating_text = await rating_elem.text_content() or ''
                        rating_match = re.search(r'([\d,\.]+)', rating_text)
                        if rating_match:
                            result['rating'] = rating_match.group(1).replace(',', '.')
                            print(f"[DEBUG] 评分: {result['rating']} (from reviewsMedley)", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 提取评分失败: {e}", file=sys.stderr)

            # 4. 提取评论数 (与排名监控爬虫保持一致)
            try:
                reviews_elem = page.locator('#acrCustomerReviewText').first
                if await reviews_elem.count() > 0:
                    reviews_text = await reviews_elem.text_content() or ''
                    # 提取数字，处理各种格式如 "2,989", "2.989", "2989"
                    num_match = re.search(r'([\d,.\s]+)', reviews_text)
                    if num_match:
                        num_str = num_match.group(1).strip()
                        # 移除空格和千位分隔符
                        num_str = re.sub(r'[\s,.]', '', num_str)
                        if num_str.isdigit():
                            result['review_count'] = int(num_str)
                            print(f"[DEBUG] 评论数: {result['review_count']}", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 提取评论数失败: {e}", file=sys.stderr)

            # 5. 提取 BSR 排名（包含类目名称）
            try:
                # 正则表达式：同时捕获排名数字和类目名称
                # 各语言格式:
                # 法语: "7 en Fraises à ongles électriques"
                # 英语: "#7 in Nail Files"
                # 德语: "Nr. 7 in Elektrische Nagelfeilen"
                # 意大利语: "n. 7 in Lime elettriche per unghie"
                # 西班牙语: "nº 7 en Limas de uñas eléctricas"
                # 日语: "ビューティー - 116,737位" 或 "ヘアカーリングワンド - 11位"
                bsr_with_category_patterns = [
                    # 法语: 数字 en 类目
                    r'(\d[\d.,]*)\s+en\s+([A-ZÀ-Úa-zà-ú][^\n\(#]{3,80})',
                    # 英语/美国: #数字 in 类目
                    r'#\s*(\d[\d.,]*)\s+in\s+([A-Za-zÀ-ú][^\n\(#]{3,80})',
                    # 英语/英国: 数字 in 类目（无 # 前缀）
                    r'(\d[\d,]*)\s+in\s+([A-Za-z][^\n\(#]{3,80})',
                    # 德语: Nr. 数字 in 类目
                    r'Nr\.?\s*(\d[\d.,]*)\s+in\s+([A-ZÄÖÜa-zäöüß][^\n\(#]{3,80})',
                    # 意大利语: n. 数字 in 类目
                    r'[nN]\.?\s*(\d[\d.,]*)\s+in\s+([A-Za-zÀ-ú][^\n\(#]{3,80})',
                    # 西班牙语: nº 数字 en 类目
                    r'[nN][º°]?\s*(\d[\d.,]*)\s+en\s+([A-Za-zÀ-ú][^\n\(#]{3,80})',
                ]

                # 日本站特殊处理：格式为 "类目 - 数字位"
                # 注意：类目名可以包含长音符号 ー，分隔符是空格包围的短横线 " - "
                jp_bsr_pattern = r'([ぁ-んァ-ヶー一-龥a-zA-Z&]+)\s+[-‐−]\s+(\d[\d,]*)\s*位'

                def extract_bsr_with_category(text, is_japan=False):
                    """从文本中提取所有 BSR 排名及其类目，返回排名最小的那个"""
                    all_bsr = []

                    # 日本站特殊处理：格式为 "类目 - 数字位"
                    if is_japan:
                        jp_matches = re.findall(jp_bsr_pattern, text)
                        for category, rank_str in jp_matches:
                            num_str = re.sub(r'[\s,]', '', rank_str)
                            if num_str.isdigit():
                                category = category.strip()
                                # 清理日本站的类目名称，移除末尾的"の売れ筋ランキングを見る"等
                                category = re.sub(r'\s*[（\(].*[）\)].*$', '', category)
                                category = category.strip()
                                print(f"[DEBUG] BSR 匹配(JP): rank={num_str}, 类目='{category}'", file=sys.stderr)
                                if len(category) >= 2:
                                    all_bsr.append((int(num_str), category))

                    # 其他国家的处理
                    for pattern in bsr_with_category_patterns:
                        matches = re.findall(pattern, text, re.IGNORECASE | re.MULTILINE)
                        for rank_str, category in matches:
                            num_str = re.sub(r'[\s,.]', '', rank_str)
                            if num_str.isdigit():
                                original_category = category.strip()
                                category = original_category

                                # 优先提取 "nella categoria X" 中的 X（意大利语）
                                nella_match = re.search(r'nella categoria\s+(.+)$', category, re.IGNORECASE)
                                if nella_match:
                                    category = nella_match.group(1).strip()
                                else:
                                    # 提取 "in der Kategorie X" 中的 X（德语）
                                    der_match = re.search(r'in der Kategorie\s+(.+)$', category, re.IGNORECASE)
                                    if der_match:
                                        category = der_match.group(1).strip()
                                    else:
                                        # 清理 "Top 100 in X" -> "X"
                                        category = re.sub(r'^Top\s+\d+\s+in\s+', '', category, flags=re.IGNORECASE)
                                        # 清理尾部的 "Voir/See/Visualizza..."
                                        category = re.sub(r'\s*(Voir|See|Mehr|Ver|Vedi|Visualizza).*$', '', category, flags=re.IGNORECASE)

                                category = category.strip()
                                print(f"[DEBUG] BSR 匹配: rank={num_str}, 原始类目='{original_category}' -> 清理后='{category}'", file=sys.stderr)
                                if len(category) > 3:
                                    all_bsr.append((int(num_str), category))

                    if all_bsr:
                        print(f"[DEBUG] BSR 所有匹配: {all_bsr}", file=sys.stderr)
                        # 返回排名最小的（子类目排名通常最小）
                        all_bsr.sort(key=lambda x: x[0])
                        return all_bsr[0]
                    return None

                # 方法1: 从特定 ID 的产品详情表格提取
                bsr_id_selectors = [
                    '#productDetails_detailBullets_sections1',
                    '#detailBulletsWrapper_feature_div',
                    '#prodDetails',
                    '#detailBullets_feature_div',
                ]
                is_japan = country == 'JP'
                for selector in bsr_id_selectors:
                    try:
                        detail_elem = page.locator(selector).first
                        if await detail_elem.count() > 0:
                            detail_text = await detail_elem.inner_text()
                            bsr_info = extract_bsr_with_category(detail_text, is_japan=is_japan)
                            if bsr_info:
                                rank, category = bsr_info
                                result['bsr_rank'] = f"#{rank} in {category}"
                                print(f"[DEBUG] BSR: {result['bsr_rank']} (from {selector})", file=sys.stderr)
                                break
                    except Exception as e:
                        print(f"[DEBUG] BSR选择器 {selector} 失败: {e}", file=sys.stderr)
                        continue

                # 方法2: 遍历所有 .prodDetTable 表格
                if not result['bsr_rank']:
                    try:
                        tables = page.locator('.prodDetTable')
                        table_count = await tables.count()
                        print(f"[DEBUG] 找到 {table_count} 个 prodDetTable", file=sys.stderr)
                        for i in range(table_count):
                            table = tables.nth(i)
                            table_text = await table.inner_text()
                            bsr_info = extract_bsr_with_category(table_text, is_japan=is_japan)
                            if bsr_info:
                                rank, category = bsr_info
                                result['bsr_rank'] = f"#{rank} in {category}"
                                print(f"[DEBUG] BSR: {result['bsr_rank']} (from table {i})", file=sys.stderr)
                                break
                    except Exception as e:
                        print(f"[DEBUG] 遍历表格失败: {e}", file=sys.stderr)

                # 方法3: 如果上面没找到，尝试从整个页面搜索
                if not result['bsr_rank']:
                    page_content = await page.content()
                    bsr_info = extract_bsr_with_category(page_content, is_japan=is_japan)
                    if bsr_info:
                        rank, category = bsr_info
                        result['bsr_rank'] = f"#{rank} in {category}"
                        print(f"[DEBUG] BSR (页面): {result['bsr_rank']}", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 提取BSR失败: {e}", file=sys.stderr)

            # 5.5 提取上架时间 (Date First Available)
            try:
                # 各语言格式:
                # 英语: "Date First Available : January 1, 2020" 或 "Date First Available: 1 Jan. 2020"
                # 德语: "Im Angebot von Amazon.de seit : 1. Januar 2020"
                # 法语: "Date de mise en ligne sur Amazon.fr : 1 janvier 2020"
                # 意大利语: "Disponibile su Amazon.it a partire dal : 1 gennaio 2020"
                # 西班牙语: "Fecha de disponibilidad en Amazon.es : 1 de enero de 2020"
                # 日语: "Amazon.co.jpでの取り扱い開始日 : 2020/1/1"
                date_patterns = [
                    # 英语 (US/UK/CA/AU)
                    r'Date First Available\s*[:\-]?\s*([A-Za-z]+\.?\s*\d{1,2},?\s*\d{4}|\d{1,2}\s+[A-Za-z]+\.?\s*\d{4})',
                    # 德语
                    r'Im Angebot von Amazon\.de seit\s*[:\-]?\s*(\d{1,2}\.?\s*[A-Za-zäöüÄÖÜß]+\.?\s*\d{4})',
                    r'Erstmals im Angebot auf Amazon\.de\s*[:\-]?\s*(\d{1,2}\.?\s*[A-Za-zäöüÄÖÜß]+\.?\s*\d{4})',
                    # 法语
                    r'Date de mise en ligne sur Amazon\.fr\s*[:\-]?\s*(\d{1,2}\s+[a-zéèàùâêîôûäëïöü]+\.?\s*\d{4})',
                    r'Disponible sur Amazon\.fr depuis le\s*[:\-]?\s*(\d{1,2}\s+[a-zéèàùâêîôûäëïöü]+\.?\s*\d{4})',
                    # 意大利语
                    r'Disponibile su Amazon\.it\s+(?:a partire\s+)?dal\s*[:\-]?\s*(\d{1,2}\s+[a-z]+\.?\s*\d{4})',
                    # 西班牙语 - "Producto en Amazon.es desde : 18 febrero 2023"
                    r'Producto en Amazon\.es desde\s*[:\-‏‎\s]*(\d{1,2}\s+(?:de\s+)?[a-z]+(?:\s+de)?\s+\d{4})',
                    r'Fecha de disponibilidad en Amazon\.es\s*[:\-]?\s*(\d{1,2}\s+(?:de\s+)?[a-z]+(?:\s+de)?\s+\d{4})',
                    # 日语 - 注意 Amazon.co.jp 和 での 之间可能有空格
                    r'Amazon\.co\.jp\s*での取り扱い開始日\s*[:\-]?\s*(\d{4}[/年]\d{1,2}[/月]\d{1,2}日?)',
                    # 日语备用格式
                    r'取り扱い開始日\s*[:\-]?\s*(\d{4}[/年]\d{1,2}[/月]\d{1,2}日?)',
                ]

                def extract_date_first_available(text):
                    for pattern in date_patterns:
                        match = re.search(pattern, text, re.IGNORECASE)
                        if match:
                            date_str = match.group(1).strip()
                            print(f"[DEBUG] 上架时间匹配: '{date_str}'", file=sys.stderr)
                            return date_str
                    return None

                # 从产品详情表格提取
                for selector in bsr_id_selectors:
                    try:
                        detail_elem = page.locator(selector).first
                        if await detail_elem.count() > 0:
                            detail_text = await detail_elem.inner_text()
                            date_str = extract_date_first_available(detail_text)
                            if date_str:
                                result['date_first_available'] = date_str
                                print(f"[DEBUG] 上架时间: {date_str} (from {selector})", file=sys.stderr)
                                break
                    except:
                        continue

                # 如果没找到，尝试从整个页面搜索
                if not result['date_first_available']:
                    page_content = await page.content()
                    date_str = extract_date_first_available(page_content)
                    if date_str:
                        result['date_first_available'] = date_str
                        print(f"[DEBUG] 上架时间 (页面): {date_str}", file=sys.stderr)

            except Exception as e:
                print(f"[DEBUG] 提取上架时间失败: {e}", file=sys.stderr)

            # 6. 提取五点描述 (Bullet Points)
            try:
                # 尝试多个选择器（从最精确到最宽泛）
                bullet_selectors = [
                    '#feature-bullets ul li span.a-list-item',
                    '#featurebullets_feature_div ul li span.a-list-item',
                    '#productFactsDesktopExpander ul li span.a-list-item',
                    # 备用：关于此商品区域
                    '#productFactsDesktop_feature_div ul li',
                ]
                for selector in bullet_selectors:
                    bullets_elem = page.locator(selector)
                    count = await bullets_elem.count()
                    print(f"[DEBUG] 尝试选择器 {selector}: 找到 {count} 个", file=sys.stderr)
                    if count > 0:
                        bullets_list = await bullets_elem.all()
                        for bullet in bullets_list:
                            text = await bullet.text_content()
                            if text:
                                text = text.strip()
                                # 清理多余空白
                                text = re.sub(r'\s+', ' ', text)
                                # 过滤掉无关内容
                                if (text and len(text) > 15
                                    and not text.startswith('Make sure')
                                    and not text.startswith('See more')
                                    and 'out of 5 stars' not in text
                                    and not text.startswith('Product Dimensions')
                                    and not text.startswith('Item model')
                                    and not text.startswith('Manufacturer')
                                    and not text.startswith('ASIN')
                                    and not text.startswith('Customer Reviews')
                                    and 'Reviewed in' not in text
                                    and 'star' not in text[:20]):
                                    result['bullets'].append(text)
                        if result['bullets']:
                            print(f"[DEBUG] 五点描述: {len(result['bullets'])} 条", file=sys.stderr)
                            break

                # 如果上面没找到，尝试从页面其他区域获取（宽泛选择器）
                if not result['bullets']:
                    # 之前能用的选择器
                    broad_bullets = page.locator('ul.a-unordered-list.a-vertical li span.a-list-item')
                    count = await broad_bullets.count()
                    print(f"[DEBUG] 宽泛选择器: 找到 {count} 个", file=sys.stderr)
                    if count > 0:
                        for i in range(min(count, 30)):  # 最多检查30个
                            item = broad_bullets.nth(i)
                            text = await item.text_content()
                            if text:
                                text = re.sub(r'\s+', ' ', text.strip())
                                # 过滤掉明显无关的内容
                                skip_keywords = [
                                    'out of 5 stars', 'Reviewed in',
                                    'Product Dimensions', 'Item model',
                                    'ASIN :', 'Customer Reviews:',
                                    'P.when(', 'function(A)', 'execute(',
                                    'dpAcrHas', '63%18%', 'star4 star',
                                    '<img src=', 'Verified Purchase'
                                ]
                                # 文本长度在合理范围且不包含脚本代码
                                if (30 < len(text) < 400
                                    and not any(skip in text for skip in skip_keywords)
                                    and 'var ' not in text
                                    and 'P.when' not in text):
                                    result['bullets'].append(text)
                                    print(f"[DEBUG] 保留五点 {len(result['bullets'])}: {text[:50]}...", file=sys.stderr)
                                    if len(result['bullets']) >= 7:  # 最多7条
                                        break
                        if result['bullets']:
                            print(f"[DEBUG] 五点描述 (宽泛): {len(result['bullets'])} 条", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 提取五点描述失败: {e}", file=sys.stderr)

            # 7. 提取商品描述
            try:
                # 优先获取纯文本描述，避免 A+ 内容（A+ 通常包含大量 CSS/JS）
                desc_selectors = [
                    '#productDescription p',
                    '#productDescription',
                ]
                for selector in desc_selectors:
                    desc_elem = page.locator(selector)
                    if await desc_elem.count() > 0:
                        # 使用 inner_text() 而不是 text_content()，只获取可见文本
                        desc_text = await desc_elem.inner_text()
                        if desc_text:
                            desc_text = desc_text.strip()
                            # 清理多余空白
                            desc_text = re.sub(r'\s+', ' ', desc_text)
                            # 过滤掉明显是 CSS/JS 的内容
                            if (len(desc_text) > 20
                                and not desc_text.startswith('.aplus')
                                and 'function ' not in desc_text
                                and '{' not in desc_text[:100]):
                                result['description'] = desc_text[:5000]  # 限制长度
                                print(f"[DEBUG] 描述: {len(result['description'])} 字符", file=sys.stderr)
                                break
            except Exception as e:
                print(f"[DEBUG] 提取描述失败: {e}", file=sys.stderr)

        except Exception as e:
            result['error'] = str(e)
            print(f"[DEBUG] 爬取失败: {e}", file=sys.stderr)

        finally:
            await browser.close()

    return result


async def fetch_listings_batch(items: list, headless="new") -> list:
    """
    批量爬取多个 ASIN 的 Listing 信息（单浏览器模式，更高效）

    参数:
        items: [(competitor_id, asin, country), ...]
        headless: 无头模式

    返回: [(competitor_id, result), ...]
    """
    if not items:
        return []

    results = []

    # 按国家分组，同一国家的产品用同一个浏览器会话
    from collections import defaultdict
    country_groups = defaultdict(list)
    for item in items:
        competitor_id, asin, country = item
        country_groups[country.upper()].append((competitor_id, asin))

    async with async_playwright() as p:
        # 配置浏览器启动选项
        launch_options = {}
        if headless == "new":
            launch_options["headless"] = True
            launch_options["args"] = ["--headless=new"]
        elif headless == False:
            launch_options["headless"] = False
        else:
            launch_options["headless"] = headless

        browser = await p.chromium.launch(**launch_options)

        total_idx = 0
        total_count = len(items)

        for country, country_items in country_groups.items():
            config = COUNTRY_CONFIG.get(country, COUNTRY_CONFIG["US"])

            # 国家对应的地理位置坐标
            geo_locations = {
                "DE": {"latitude": 52.5200, "longitude": 13.4050},
                "FR": {"latitude": 48.8566, "longitude": 2.3522},
                "UK": {"latitude": 51.5074, "longitude": -0.1278},
                "US": {"latitude": 40.7128, "longitude": -74.0060},
                "IT": {"latitude": 41.9028, "longitude": 12.4964},
                "ES": {"latitude": 40.4168, "longitude": -3.7038},
                "JP": {"latitude": 35.6762, "longitude": 139.6503},
                "CA": {"latitude": 43.6532, "longitude": -79.3832},
                "AU": {"latitude": -33.8688, "longitude": 151.2093},
            }

            timezones = {
                "DE": "Europe/Berlin", "FR": "Europe/Paris", "UK": "Europe/London",
                "IT": "Europe/Rome", "ES": "Europe/Madrid", "US": "America/New_York",
                "JP": "Asia/Tokyo", "CA": "America/Toronto", "AU": "Australia/Sydney",
            }

            # 为每个国家创建一个 context
            context = await browser.new_context(
                locale=config['language'],
                timezone_id=timezones.get(country, "America/New_York"),
                viewport={"width": 1920, "height": 1080},
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                geolocation=geo_locations.get(country, geo_locations["US"]),
                permissions=["geolocation"]
            )

            page = await context.new_page()
            await page.set_extra_http_headers({
                'Accept-Language': f'{config["language"]},en;q=0.9',
            })

            address_set = False  # 标记是否已设置邮编

            for competitor_id, asin in country_items:
                total_idx += 1
                print(f"[DEBUG] 批量爬取 {total_idx}/{total_count}: {asin} ({country})", file=sys.stderr)

                result = {
                    "asin": asin,
                    "country": country,
                    "title": None,
                    "price": None,
                    "rating": None,
                    "review_count": None,
                    "bsr_rank": None,
                    "date_first_available": None,
                    "image_url": None,
                    "bullets": [],
                    "description": None,
                    "fetched_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
                    "error": None
                }

                try:
                    # 访问产品详情页
                    detail_url = f"{config['base_url']}/dp/{asin}"
                    print(f"[DEBUG] 访问: {detail_url}", file=sys.stderr)
                    await page.goto(detail_url, wait_until="domcontentloaded", timeout=30000)
                    await page.wait_for_timeout(2000)

                    # 处理 Cookie 弹窗（只需要处理一次）
                    try:
                        cookie_btn = page.locator('#sp-cc-accept')
                        if await cookie_btn.is_visible(timeout=1500):
                            await cookie_btn.click()
                            await page.wait_for_timeout(1000)
                    except:
                        pass

                    # 处理"继续购物"黄色按钮
                    try:
                        continue_texts = ["Continuer les achats", "Continue shopping", "Weiter einkaufen", "Weiter shoppen", "Continua lo shopping", "Continua con gli acquisti", "Seguir comprando"]
                        clicked = False
                        for text in continue_texts:
                            try:
                                btn = page.get_by_role("button", name=text)
                                if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                                    print(f"[DEBUG] 发现继续购物按钮: {text}", file=sys.stderr)
                                    await btn.first.click()
                                    await page.wait_for_timeout(2000)
                                    clicked = True
                                    break
                            except:
                                pass
                            if not clicked:
                                try:
                                    btn = page.get_by_text(text, exact=True)
                                    if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                                        await btn.first.click()
                                        await page.wait_for_timeout(2000)
                                        clicked = True
                                        break
                                except:
                                    pass
                        if not clicked:
                            for selector in ['input[type="submit"][value*="Continuer"]', 'input[type="submit"][value*="Continue"]']:
                                try:
                                    elem = page.locator(selector).first
                                    if await elem.is_visible(timeout=500):
                                        await elem.click()
                                        await page.wait_for_timeout(2000)
                                        break
                                except:
                                    pass
                    except:
                        pass

                    # 设置配送地址（每个国家只需设置一次）
                    if not address_set:
                        try:
                            address_success, address_text = await set_delivery_address(page, country, config['zipcode'], max_retries=3)
                            if address_success:
                                print(f"[DEBUG] 配送地址设置成功: {address_text}", file=sys.stderr)
                                address_set = True
                                # 刷新页面
                                await page.reload(wait_until="domcontentloaded", timeout=30000)
                                await page.wait_for_timeout(2000)

                                # 刷新后可能再次出现"继续购物"按钮
                                try:
                                    refresh_continue_texts = [
                                        "Continuer les achats", "Continue shopping",
                                        "Weiter einkaufen", "Weiter shoppen",
                                        "Continua lo shopping", "Continua con gli acquisti",
                                        "Seguir comprando"
                                    ]
                                    for text in refresh_continue_texts:
                                        try:
                                            btn = page.get_by_text(text, exact=True)
                                            if await btn.count() > 0 and await btn.first.is_visible(timeout=1000):
                                                print(f"[DEBUG] 刷新后再次发现继续购物按钮: {text}，点击", file=sys.stderr)
                                                await btn.first.click()
                                                await page.wait_for_timeout(2000)
                                                break
                                        except:
                                            continue
                                except:
                                    pass
                        except Exception as e:
                            print(f"[DEBUG] 设置配送地址出错: {e}", file=sys.stderr)

                        # 设置货币（处理德国站等可能显示美元的情况）
                        try:
                            await set_currency(page, country)
                        except Exception as e:
                            print(f"[DEBUG] 设置货币出错: {e}", file=sys.stderr)

                    # 提取产品信息（与单个爬取一致）
                    # 1. 标题 (使用 span#productTitle 避免匹配到 hidden input)
                    try:
                        title_elem = page.locator('span#productTitle').first
                        title = await title_elem.text_content(timeout=5000)
                        result['title'] = title.strip() if title else None
                    except:
                        pass

                    # 1.5 图片
                    try:
                        for img_sel in ['#landingImage', '#imgBlkFront', '#main-image']:
                            img_elem = page.locator(img_sel).first
                            if await img_elem.count() > 0:
                                img_url = await img_elem.get_attribute('src')
                                if img_url and img_url.startswith('http'):
                                    result['image_url'] = img_url
                                    break
                    except:
                        pass

                    # 2. 价格
                    try:
                        price_elem = page.locator('.a-price .a-offscreen').first
                        if await page.locator('.a-price .a-offscreen').count() > 0:
                            price = await price_elem.text_content()
                            if price:
                                result['price'] = price.strip()
                    except:
                        pass

                    # 3. 评分 (使用精确选择器)
                    try:
                        rating_selectors = [
                            '#acrPopover .a-icon-alt',
                            '#averageCustomerReviews .a-icon-alt',
                            '#acrPopover span.a-icon-alt',
                        ]
                        for selector in rating_selectors:
                            try:
                                rating_elem = page.locator(selector).first
                                if await rating_elem.count() > 0:
                                    rating_text = await rating_elem.text_content() or ''
                                    rating_match = re.search(r'([\d,\.]+)', rating_text)
                                    if rating_match:
                                        result['rating'] = rating_match.group(1).replace(',', '.')
                                        break
                            except:
                                continue
                    except:
                        pass

                    # 4. 评论数
                    try:
                        reviews_elem = page.locator('#acrCustomerReviewText').first
                        if await reviews_elem.count() > 0:
                            reviews_text = await reviews_elem.text_content() or ''
                            num_match = re.search(r'([\d,.\s]+)', reviews_text)
                            if num_match:
                                num_str = re.sub(r'[\s,.]', '', num_match.group(1).strip())
                                if num_str.isdigit():
                                    result['review_count'] = int(num_str)
                    except:
                        pass

                    # 5. BSR (提取子类目排名及类目名称)
                    try:
                        # 正则表达式：同时捕获排名数字和类目名称
                        bsr_with_category_patterns = [
                            r'(\d[\d.,]*)\s+en\s+([A-ZÀ-Úa-zà-ú][^\n\(#]{3,80})',
                            r'#\s*(\d[\d.,]*)\s+in\s+([A-Za-zÀ-ú][^\n\(#]{3,80})',
                            r'(\d[\d,]*)\s+in\s+([A-Za-z][^\n\(#]{3,80})',  # 英国格式
                            r'Nr\.?\s*(\d[\d.,]*)\s+in\s+([A-ZÄÖÜa-zäöüß][^\n\(#]{3,80})',
                            r'[nN]\.?\s*(\d[\d.,]*)\s+in\s+([A-Za-zÀ-ú][^\n\(#]{3,80})',
                            r'[nN][º°]?\s*(\d[\d.,]*)\s+en\s+([A-Za-zÀ-ú][^\n\(#]{3,80})',
                        ]

                        # 日本站特殊处理：格式为 "类目 - 数字位"
                        # 注意：类目名可以包含长音符号 ー，分隔符是空格包围的短横线 " - "
                        jp_bsr_pattern = r'([ぁ-んァ-ヶー一-龥a-zA-Z&]+)\s+[-‐−]\s+(\d[\d,]*)\s*位'
                        is_japan = country == 'JP'

                        def extract_bsr_batch(text):
                            all_bsr = []

                            # 日本站特殊处理
                            if is_japan:
                                jp_matches = re.findall(jp_bsr_pattern, text)
                                for category, rank_str in jp_matches:
                                    num_str = re.sub(r'[\s,]', '', rank_str)
                                    if num_str.isdigit():
                                        category = category.strip()
                                        category = re.sub(r'\s*[（\(].*[）\)].*$', '', category)
                                        category = category.strip()
                                        if len(category) >= 2:
                                            all_bsr.append((int(num_str), category))

                            for pattern in bsr_with_category_patterns:
                                matches = re.findall(pattern, text, re.IGNORECASE | re.MULTILINE)
                                for rank_str, category in matches:
                                    num_str = re.sub(r'[\s,.]', '', rank_str)
                                    if num_str.isdigit():
                                        category = category.strip()
                                        # 优先提取 "nella categoria X" 中的 X（意大利语）
                                        nella_match = re.search(r'nella categoria\s+(.+)$', category, re.IGNORECASE)
                                        if nella_match:
                                            category = nella_match.group(1).strip()
                                        else:
                                            # 提取 "in der Kategorie X" 中的 X（德语）
                                            der_match = re.search(r'in der Kategorie\s+(.+)$', category, re.IGNORECASE)
                                            if der_match:
                                                category = der_match.group(1).strip()
                                            else:
                                                # 清理 "Top 100 in X" -> "X"
                                                category = re.sub(r'^Top\s+\d+\s+in\s+', '', category, flags=re.IGNORECASE)
                                                # 清理尾部的 "Voir/See/Visualizza..."
                                                category = re.sub(r'\s*(Voir|See|Mehr|Ver|Vedi|Visualizza).*$', '', category, flags=re.IGNORECASE)
                                        category = category.strip()
                                        if len(category) > 3:
                                            all_bsr.append((int(num_str), category))
                            if all_bsr:
                                all_bsr.sort(key=lambda x: x[0])
                                return all_bsr[0]
                            return None

                        bsr_id_selectors = ['#productDetails_detailBullets_sections1', '#detailBulletsWrapper_feature_div', '#prodDetails']
                        for selector in bsr_id_selectors:
                            try:
                                detail_elem = page.locator(selector).first
                                if await detail_elem.count() > 0:
                                    detail_text = await detail_elem.inner_text()
                                    bsr_info = extract_bsr_batch(detail_text)
                                    if bsr_info:
                                        rank, category = bsr_info
                                        result['bsr_rank'] = f"#{rank} in {category}"
                                        break
                            except:
                                continue
                        # 方法2: 遍历所有表格
                        if not result['bsr_rank']:
                            tables = page.locator('.prodDetTable')
                            table_count = await tables.count()
                            for i in range(table_count):
                                try:
                                    table = tables.nth(i)
                                    table_text = await table.inner_text()
                                    bsr_info = extract_bsr_batch(table_text)
                                    if bsr_info:
                                        rank, category = bsr_info
                                        result['bsr_rank'] = f"#{rank} in {category}"
                                        break
                                except:
                                    continue
                    except:
                        pass

                    # 5.5 上架时间
                    try:
                        date_patterns = [
                            r'Date First Available\s*[:\-]?\s*([A-Za-z]+\.?\s*\d{1,2},?\s*\d{4}|\d{1,2}\s+[A-Za-z]+\.?\s*\d{4})',
                            r'Im Angebot von Amazon\.de seit\s*[:\-]?\s*(\d{1,2}\.?\s*[A-Za-zäöüÄÖÜß]+\.?\s*\d{4})',
                            r'Erstmals im Angebot auf Amazon\.de\s*[:\-]?\s*(\d{1,2}\.?\s*[A-Za-zäöüÄÖÜß]+\.?\s*\d{4})',
                            r'Date de mise en ligne sur Amazon\.fr\s*[:\-]?\s*(\d{1,2}\s+[a-zéèàùâêîôûäëïöü]+\.?\s*\d{4})',
                            r'Disponible sur Amazon\.fr depuis le\s*[:\-]?\s*(\d{1,2}\s+[a-zéèàùâêîôûäëïöü]+\.?\s*\d{4})',
                            r'Disponibile su Amazon\.it\s+(?:a partire\s+)?dal\s*[:\-]?\s*(\d{1,2}\s+[a-z]+\.?\s*\d{4})',
                            r'Producto en Amazon\.es desde\s*[:\-‏‎\s]*(\d{1,2}\s+(?:de\s+)?[a-z]+(?:\s+de)?\s+\d{4})',
                            r'Fecha de disponibilidad en Amazon\.es\s*[:\-]?\s*(\d{1,2}\s+(?:de\s+)?[a-z]+(?:\s+de)?\s+\d{4})',
                            r'Amazon\.co\.jp\s*での取り扱い開始日\s*[:\-]?\s*(\d{4}[/年]\d{1,2}[/月]\d{1,2}日?)',
                            r'取り扱い開始日\s*[:\-]?\s*(\d{4}[/年]\d{1,2}[/月]\d{1,2}日?)',
                        ]
                        bsr_selectors = ['#productDetails_detailBullets_sections1', '#detailBulletsWrapper_feature_div', '#prodDetails']
                        for selector in bsr_selectors:
                            try:
                                detail_elem = page.locator(selector).first
                                if await detail_elem.count() > 0:
                                    detail_text = await detail_elem.inner_text()
                                    for pattern in date_patterns:
                                        match = re.search(pattern, detail_text, re.IGNORECASE)
                                        if match:
                                            result['date_first_available'] = match.group(1).strip()
                                            break
                                    if result['date_first_available']:
                                        break
                            except:
                                continue
                    except:
                        pass

                    # 6. 五点描述
                    try:
                        bullet_selectors = [
                            '#feature-bullets ul li span.a-list-item',
                            '#featurebullets_feature_div ul li span.a-list-item',
                            '#productFactsDesktopExpander ul li span.a-list-item',
                        ]
                        for selector in bullet_selectors:
                            bullets_elem = page.locator(selector)
                            count = await bullets_elem.count()
                            if count > 0:
                                bullets_list = await bullets_elem.all()
                                for bullet in bullets_list:
                                    text = await bullet.text_content()
                                    if text:
                                        text = re.sub(r'\s+', ' ', text.strip())
                                        if (text and len(text) > 15
                                            and not text.startswith('Make sure')
                                            and 'out of 5 stars' not in text):
                                            result['bullets'].append(text)
                                if result['bullets']:
                                    break
                    except:
                        pass

                    # 7. 描述
                    try:
                        desc_selectors = ['#productDescription p', '#productDescription']
                        for selector in desc_selectors:
                            desc_elem = page.locator(selector)
                            if await desc_elem.count() > 0:
                                desc_text = await desc_elem.text_content()
                                if desc_text:
                                    desc_text = re.sub(r'\s+', ' ', desc_text.strip())
                                    if len(desc_text) > 20:
                                        result['description'] = desc_text[:5000]
                                        break
                    except:
                        pass

                    print(f"[DEBUG] 完成: {asin} - 标题: {result['title'][:30] if result['title'] else '无'}...", file=sys.stderr)

                except Exception as e:
                    result['error'] = str(e)
                    print(f"[DEBUG] 爬取失败: {asin} - {e}", file=sys.stderr)

                results.append((competitor_id, result))

                # 输出进度
                progress = {
                    "type": "progress",
                    "competitor_id": competitor_id,
                    "result": result
                }
                output = json.dumps(progress, ensure_ascii=False)
                sys.stdout.buffer.write(output.encode('utf-8'))
                sys.stdout.buffer.write(b'\n')
                sys.stdout.flush()

                # 产品间隔（同一浏览器内切换页面，间隔可以短一些）
                if total_idx < total_count:
                    await page.wait_for_timeout(1500)

            # 关闭该国家的 context
            await context.close()

        await browser.close()

    return results


def main():
    if len(sys.argv) < 2:
        print(json.dumps({
            "error": "用法: python amazon_listing_crawler.py <asin> <country> [headless]\n或: python amazon_listing_crawler.py --batch [headless]"
        }))
        sys.exit(1)

    # 批量模式：从stdin读取JSON
    if sys.argv[1] == '--batch':
        headless_arg = sys.argv[2].lower() if len(sys.argv) > 2 else "new"
        if headless_arg == "new":
            headless = "new"
        elif headless_arg == "false":
            headless = False
        else:
            headless = True

        # 从stdin读取JSON: [(competitor_id, asin, country), ...]
        input_data = sys.stdin.read()
        items = json.loads(input_data)

        results = asyncio.run(fetch_listings_batch(items, headless))

        # 输出完成标记
        complete = {"type": "complete", "total": len(results)}
        output = json.dumps(complete, ensure_ascii=False)
        sys.stdout.buffer.write(output.encode('utf-8'))
        sys.stdout.buffer.write(b'\n')
        sys.exit(0)

    # 单个 ASIN 模式
    if len(sys.argv) < 3:
        print(json.dumps({
            "error": "用法: python amazon_listing_crawler.py <asin> <country> [headless]"
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

    result = asyncio.run(fetch_listing_info(asin, country, headless))

    output = json.dumps(result, ensure_ascii=False)
    sys.stdout.buffer.write(output.encode('utf-8'))
    sys.stdout.buffer.write(b'\n')


if __name__ == "__main__":
    main()
