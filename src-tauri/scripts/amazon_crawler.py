#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Amazon 关键词排名爬虫
使用 cloudscraper 绕过反爬虫机制
"""

import sys
import json
import random
import time
import re
from urllib.parse import quote_plus
from datetime import datetime, timezone

try:
    import cloudscraper
    from bs4 import BeautifulSoup
except ImportError as e:
    print(json.dumps({
        "error": f"缺少依赖库: {e}. 请运行: pip install cloudscraper beautifulsoup4"
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
    },
    "JP": {
        "base_url": "https://www.amazon.co.jp",
        "language": "ja-JP",
        "zipcode": "100-0001",
        "market_param": "__mk_ja_JP=%E3%82%AB%E3%82%BF%E3%82%AB%E3%83%8A",
        "currency": "JPY"
    }
}

USER_AGENTS = [
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15',
]


def get_random_user_agent():
    return random.choice(USER_AGENTS)


def search_keyword(keyword: str, target_asin: str, country: str, max_pages: int = 3, proxy: str = None) -> dict:
    """
    搜索关键词并返回目标ASIN的排名

    参数:
        proxy: 代理地址，格式如 "http://127.0.0.1:7897" 或 "socks5://127.0.0.1:7891"
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
        "product_info": None,
        "organic_top_50": [],
        "sponsored_top_20": [],
        "checked_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "error": None,
        "warning": None  # 警告信息（如地理限制）
    }

    # 构建搜索URL
    encoded_keyword = quote_plus(keyword)
    market_param = f"&{config['market_param']}" if config['market_param'] else ""
    search_url = f"{config['base_url']}/s?k={encoded_keyword}{market_param}"

    headers = {
        'User-Agent': get_random_user_agent(),
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8',
        'Accept-Language': f"{config['language']},{config['language'].split('-')[0]};q=0.9",
        'Accept-Encoding': 'gzip, deflate, br',
        'Connection': 'keep-alive',
        'Upgrade-Insecure-Requests': '1',
        'Sec-Fetch-Dest': 'document',
        'Sec-Fetch-Mode': 'navigate',
        'Sec-Fetch-Site': 'same-origin',
        'Sec-Fetch-User': '?1',
        'Cache-Control': 'max-age=0',
        'Referer': config['base_url'] + '/',
    }

    organic_position = 0
    sponsored_position = 0
    found_organic = False
    found_sponsored = False

    try:
        # 创建 cloudscraper session
        session = cloudscraper.create_scraper(
            browser={
                'browser': 'chrome',
                'platform': 'windows',
                'desktop': True
            }
        )

        # 配置代理
        if proxy:
            session.proxies = {
                'http': proxy,
                'https': proxy
            }
            print(f"[DEBUG] 使用代理: {proxy}", file=sys.stderr)

        # 设置货币偏好和位置相关 cookie（强制显示当地货币和商品）
        currency = config.get('currency', 'USD')
        domain = config['base_url'].replace('https://', '').replace('http://', '')
        session.cookies.set('i18n-prefs', currency, domain=domain)
        session.cookies.set('lc-main', config['language'].replace('-', '_'), domain=domain)

        # 设置位置相关的 cookie（帮助 Amazon 识别目标市场）
        # ubid-acb* 是 Amazon 的会话 cookie，sp-cdn 是 CDN 位置 cookie
        if country == 'DE':
            session.cookies.set('sp-cdn', '"L5Z9:DE"', domain=domain)
        elif country == 'FR':
            session.cookies.set('sp-cdn', '"L5Z9:FR"', domain=domain)
        elif country == 'UK':
            session.cookies.set('sp-cdn', '"L5Z9:GB"', domain=domain)

        # 步骤1: 访问主页建立session
        try:
            home_response = session.get(
                config['base_url'],
                headers=headers,
                timeout=15
            )
        except Exception as e:
            result["error"] = f"访问主页失败: {e}"
            return result

        # 步骤2: 设置配送地址（关键步骤！模拟用户在页面左上角选择配送地址）
        # 尝试多个API端点，直到成功为止
        address_set_success = False
        address_endpoints = [
            # 端点1: 标准地址更改API (form-data格式)
            {
                'url': f"{config['base_url']}/gp/delivery/ajax/address-change.html",
                'data': {
                    'locationType': 'LOCATION_INPUT',
                    'zipCode': config['zipcode'],
                    'deviceType': 'web',
                    'pageType': 's',
                    'actionSource': 'glow',
                },
                'content_type': 'application/x-www-form-urlencoded',
                'use_json': False,
            },
            # 端点2: Portal migration API (JSON格式)
            {
                'url': f"{config['base_url']}/portal-migration/hz/glow/address-change",
                'data': {
                    'locationType': 'LOCATION_INPUT',
                    'zipCode': config['zipcode'],
                    'storeContext': 'generic',
                    'deviceType': 'web',
                    'pageType': 's',
                    'actionSource': 'glow',
                },
                'content_type': 'application/json',
                'use_json': True,
            },
        ]

        for endpoint in address_endpoints:
            if address_set_success:
                break
            try:
                address_headers = headers.copy()
                address_headers.update({
                    'Content-Type': endpoint['content_type'],
                    'X-Requested-With': 'XMLHttpRequest',
                    'Referer': config['base_url'] + '/',
                    'Origin': config['base_url'],
                })

                # 先尝试不跟随重定向（避免SSL错误），如果失败再尝试跟随
                try:
                    if endpoint['use_json']:
                        address_response = session.post(
                            endpoint['url'],
                            json=endpoint['data'],
                            headers=address_headers,
                            timeout=15,
                            allow_redirects=False  # 先尝试不跟随重定向
                        )
                    else:
                        address_response = session.post(
                            endpoint['url'],
                            data=endpoint['data'],
                            headers=address_headers,
                            timeout=15,
                            allow_redirects=False
                        )
                except Exception as first_error:
                    # 如果第一次失败，尝试跟随重定向
                    print(f"[DEBUG] 首次请求失败，尝试跟随重定向: {first_error}", file=sys.stderr)
                    if endpoint['use_json']:
                        address_response = session.post(
                            endpoint['url'],
                            json=endpoint['data'],
                            headers=address_headers,
                            timeout=15
                        )
                    else:
                        address_response = session.post(
                            endpoint['url'],
                            data=endpoint['data'],
                            headers=address_headers,
                            timeout=15
                        )

                # 检查API返回值
                if address_response.status_code == 200:
                    try:
                        address_result = address_response.json()
                        # 调试日志
                        print(f"[DEBUG] 邮编API返回 ({country}): isAddressUpdated={address_result.get('isAddressUpdated')}", file=sys.stderr)
                        if address_result.get('isAddressUpdated') == 1:
                            address_set_success = True
                            print(f"[DEBUG] 配送地址设置成功: {config['zipcode']} ({country})", file=sys.stderr)
                    except Exception as e:
                        print(f"[DEBUG] 解析邮编API响应失败: {e}", file=sys.stderr)
                elif address_response.status_code in [301, 302, 303, 307, 308]:
                    # 重定向响应 - 可能需要跟随重定向，但Amazon内部服务器不可访问
                    print(f"[DEBUG] 邮编API重定向到: {address_response.headers.get('Location', 'unknown')} ({country})", file=sys.stderr)
                    # 尝试跟随重定向获取cookie
                    try:
                        redirect_url = address_response.headers.get('Location', '')
                        if redirect_url and not 'hp-shoppingportal' in redirect_url:
                            # 只跟随非内部服务器的重定向
                            session.get(redirect_url, headers=address_headers, timeout=10)
                    except:
                        pass
                else:
                    print(f"[DEBUG] 邮编API返回状态码: {address_response.status_code} ({country})", file=sys.stderr)
            except Exception as e:
                print(f"[DEBUG] 邮编API请求异常: {e} ({country})", file=sys.stderr)
                continue  # 尝试下一个端点

        if not address_set_success:
            # 所有端点都失败，添加警告
            result['warning'] = f'配送地址设置可能未生效（邮编: {config["zipcode"]}），排名结果可能不准确。建议使用目标市场的VPN。'

        # 等待一下
        time.sleep(random.uniform(1, 2))

        # 爬取搜索结果页面
        for page in range(1, max_pages + 1):
            if found_organic and found_sponsored:
                break

            page_url = f"{search_url}&page={page}" if page > 1 else search_url

            # 503 错误重试逻辑
            max_retries = 3
            retry_delay = 5
            response = None

            for retry in range(max_retries):
                try:
                    response = session.get(
                        page_url,
                        headers=headers,
                        timeout=30,
                        allow_redirects=True
                    )

                    if response.status_code == 503:
                        print(f"[DEBUG] 收到503错误，等待{retry_delay}秒后重试 ({retry+1}/{max_retries}) ({country})", file=sys.stderr)
                        if retry < max_retries - 1:
                            time.sleep(retry_delay)
                            retry_delay *= 2  # 指数退避
                            continue
                        else:
                            result["error"] = "Amazon 返回 503 错误，请稍后再试（已重试3次）"
                            break
                    else:
                        break  # 成功获取响应
                except Exception as e:
                    print(f"[DEBUG] 请求异常: {e}，重试 ({retry+1}/{max_retries}) ({country})", file=sys.stderr)
                    if retry < max_retries - 1:
                        time.sleep(retry_delay)
                        retry_delay *= 2
                        continue
                    raise

            if response is None or response.status_code == 503:
                break

            if response.status_code != 200:
                result["error"] = f"HTTP 错误: {response.status_code}"
                break

            html = response.text

            # 检查机器人验证
            if 'Robot Check' in html or 'Type the characters' in html or 'captcha' in html.lower():
                result["error"] = "检测到验证码页面，请稍后再试"
                break

            soup = BeautifulSoup(html, 'html.parser')

            # 解析搜索结果
            search_results = soup.find_all('div', {'data-component-type': 's-search-result'})
            print(f"[DEBUG] 第{page}页找到 {len(search_results)} 个搜索结果 ({country})", file=sys.stderr)

            # 检查 Sponsored Brand 横幅广告 (SBX)
            # 这些广告出现在搜索结果顶部/中间的品牌横幅中
            if not found_organic and not found_sponsored:
                sbx_containers = soup.find_all('div', class_=re.compile(r'sbx-desktop|_bXVsd_container'))
                for sbx in sbx_containers:
                    # 在横幅中查找目标ASIN的链接
                    sbx_links = sbx.find_all('a', href=re.compile(target_asin, re.IGNORECASE))
                    if sbx_links:
                        # 找到了！这是品牌横幅广告
                        result['sponsored_rank'] = 1  # 品牌横幅通常显示在最顶部
                        result['sponsored_page'] = page
                        result['sponsored_type'] = 'brand_banner'  # 标记为品牌横幅
                        found_sponsored = True
                        print(f"[DEBUG] 在品牌横幅广告(SBX)中找到目标ASIN ({country})", file=sys.stderr)

            seen_asins = set()

            for item in search_results:
                asin = item.get('data-asin', '')

                if not asin or len(asin) != 10 or asin in seen_asins:
                    continue

                seen_asins.add(asin)

                # 检查是否是广告
                is_sponsored = bool(item.find('span', string=re.compile(r'Sponsored|Sponsorisé|Gesponsert|Sponsorizzato|Patrocinado', re.IGNORECASE)))

                if is_sponsored:
                    if not found_sponsored:
                        sponsored_position += 1

                        if len(result['sponsored_top_20']) < 20:
                            result['sponsored_top_20'].append(asin)

                        if asin.upper() == target_asin.upper():
                            result['sponsored_rank'] = sponsored_position
                            result['sponsored_page'] = page
                            found_sponsored = True
                else:
                    if not found_organic:
                        organic_position += 1

                        if len(result['organic_top_50']) < 50:
                            result['organic_top_50'].append(asin)

                        if asin.upper() == target_asin.upper():
                            result['organic_rank'] = organic_position
                            result['organic_page'] = page
                            found_organic = True
                            print(f"[DEBUG] 找到目标ASIN自然排名: 第{organic_position}名, 第{page}页 ({country})", file=sys.stderr)

                            # 提取产品信息
                            product_info = extract_product_info(item)
                            if product_info:
                                result['product_info'] = product_info

            # 页面间延迟
            if page < max_pages and not (found_organic and found_sponsored):
                time.sleep(random.uniform(2, 4))

        # 如果搜索结果中没有找到产品信息，从详情页获取
        if result['product_info'] is None:
            time.sleep(random.uniform(1, 2))
            product_info = fetch_product_detail(session, target_asin, config, headers)
            # 只有当成功获取到有效信息时才设置
            if product_info.get('title') or product_info.get('image_url') or product_info.get('price'):
                result['product_info'] = product_info

            # 检查是否有地理限制警告
            availability = product_info.get('availability', '')
            if availability:
                unavailable_keywords = [
                    'kann nicht', 'cannot be shipped', 'not available',
                    'nicht verfügbar', 'non disponible', 'non disponibile',
                    'no disponible', 'não pode ser'
                ]
                if any(kw in availability.lower() for kw in unavailable_keywords):
                    # 产品无法配送到当前位置，搜索结果可能不准确
                    # 如果已有警告，追加；否则设置新警告
                    geo_warning = '商品无法配送到当前检测位置，排名结果可能不准确。'
                    if result['warning']:
                        result['warning'] = f"{result['warning']} {geo_warning}"
                    else:
                        result['warning'] = geo_warning + '建议使用目标市场的VPN/代理获取准确排名。'

        # 如果没有找到排名，添加说明
        if result['organic_rank'] is None and result['sponsored_rank'] is None:
            if not result['error'] and not result['warning']:
                if len(result['organic_top_50']) > 0:
                    # 有搜索结果但没找到目标商品
                    result['warning'] = f'在前{len(result["organic_top_50"])}个搜索结果中未找到目标商品。'

    except Exception as e:
        result["error"] = f"爬取失败: {e}"

    return result


def extract_product_info(item) -> dict:
    """从搜索结果项中提取产品信息"""
    info = {
        "asin": item.get('data-asin', ''),
        "title": None,
        "price": None,
        "rating": None,
        "reviews_count": None,
        "image_url": None
    }

    # 标题
    title_elem = item.find('h2')
    if title_elem:
        info['title'] = title_elem.get_text(strip=True)

    # 价格 - 使用更精确的选择器
    price_container = item.find('span', class_='a-price')
    if price_container:
        # 优先从 a-price 容器内的 a-offscreen 获取完整价格
        price_elem = price_container.find('span', class_='a-offscreen')
        if price_elem:
            info['price'] = price_elem.get_text(strip=True)
        else:
            # 备选：从 a-price-whole 和 a-price-fraction 组合
            whole = price_container.find('span', class_='a-price-whole')
            fraction = price_container.find('span', class_='a-price-fraction')
            symbol = price_container.find('span', class_='a-price-symbol')
            if whole:
                price_text = whole.get_text(strip=True).rstrip(',').rstrip('.')
                if fraction:
                    price_text += ',' + fraction.get_text(strip=True)
                if symbol:
                    info['price'] = symbol.get_text(strip=True) + price_text
                else:
                    info['price'] = price_text

    # 评分
    rating_elem = item.find('span', class_='a-icon-alt')
    if rating_elem:
        rating_text = rating_elem.get_text(strip=True)
        match = re.search(r'(\d+[.,]?\d*)', rating_text)
        if match:
            try:
                info['rating'] = float(match.group(1).replace(',', '.'))
            except:
                pass

    # 评论数 - 从 aria-label 属性获取（最可靠）
    reviews_count = None

    # 方法1: 从链接的 aria-label 属性获取 (如 "220 évaluations", "3 599 avis")
    reviews_link = item.find('a', href=re.compile(r'customerReviews|#customerReviews'))
    if reviews_link:
        aria_label = reviews_link.get('aria-label', '')
        if aria_label:
            # 移除千位分隔符并提取数字
            nums = re.sub(r'[\s.,\u00a0\u202f]', '', aria_label)
            nums = re.sub(r'[^\d]', '', nums)
            if nums:
                try:
                    reviews_count = int(nums)
                except:
                    pass

    # 方法2: 从链接文本获取 (如 "(220)", "(3 599)")
    if not reviews_count and reviews_link:
        link_text = reviews_link.get_text(strip=True)
        # 移除括号和千位分隔符
        nums = re.sub(r'[\s.,\u00a0\u202f\(\)]', '', link_text)
        nums = re.sub(r'[^\d]', '', nums)
        if nums:
            try:
                reviews_count = int(nums)
            except:
                pass

    info['reviews_count'] = reviews_count

    # 图片
    img_elem = item.find('img', class_='s-image')
    if img_elem:
        info['image_url'] = img_elem.get('src')

    return info


def fetch_product_detail(session, asin: str, config: dict, headers: dict) -> dict:
    """从产品详情页获取产品信息（当搜索结果中找不到时使用）"""
    info = {
        "asin": asin,
        "title": None,
        "price": None,
        "rating": None,
        "reviews_count": None,
        "image_url": None,
        "availability": None  # 添加可用性信息
    }

    try:
        detail_url = f"{config['base_url']}/dp/{asin}"
        response = session.get(
            detail_url,
            headers=headers,
            timeout=30,
            allow_redirects=True
        )

        if response.status_code != 200:
            return info

        html = response.text

        # 检查机器人验证
        if 'Robot Check' in html or 'Type the characters' in html or 'captcha' in html.lower():
            return info

        soup = BeautifulSoup(html, 'html.parser')

        # 检查商品是否可用（是否能配送到当前位置）
        avail_elem = soup.find('div', id='availability')
        if avail_elem:
            avail_text = avail_elem.get_text(strip=True)
            info['availability'] = avail_text
            # 检测不可配送的情况
            unavailable_keywords = [
                'kann nicht', 'cannot be shipped', 'not available',
                'nicht verfügbar', 'non disponible', 'non disponibile'
            ]
            is_unavailable = any(kw in avail_text.lower() for kw in unavailable_keywords)
        else:
            is_unavailable = False

        # 标题
        title_elem = soup.find('span', id='productTitle')
        if title_elem:
            info['title'] = title_elem.get_text(strip=True)

        # 价格 - 从主价格区域获取，避免获取到运费等其他价格
        # 注意：只从确定的主价格区域获取，避免获取到轮播图中其他商品的价格
        # 优先级：corePrice_feature_div > corePriceDisplay > apex_desktop > buybox > 其他卖家
        price_containers = [
            soup.find('div', id='corePrice_feature_div'),
            soup.find('div', id='corePriceDisplay_desktop_feature_div'),
            soup.find('div', id='apex_desktop_newAccordionRow'),
            soup.find('div', id='buybox'),
            soup.find('div', id='newBuyBoxPrice'),
            soup.find('div', id='unifiedPrice_feature_div'),
        ]

        for container in price_containers:
            if container and not info['price']:
                # 在容器内查找 a-price（但要排除轮播图区域）
                # 检查是否在轮播图区域内
                if container.find_parent('div', class_=re.compile(r'p13n|sims|carousel')):
                    continue
                price_elem = container.find('span', class_='a-price')
                if price_elem:
                    price_offscreen = price_elem.find('span', class_='a-offscreen')
                    if price_offscreen:
                        info['price'] = price_offscreen.get_text(strip=True)
                        break
                    else:
                        whole = price_elem.find('span', class_='a-price-whole')
                        fraction = price_elem.find('span', class_='a-price-fraction')
                        symbol = price_elem.find('span', class_='a-price-symbol')
                        if whole:
                            price_text = whole.get_text(strip=True).rstrip(',').rstrip('.')
                            if fraction:
                                price_text += ',' + fraction.get_text(strip=True)
                            if symbol:
                                info['price'] = symbol.get_text(strip=True) + price_text
                            else:
                                info['price'] = price_text
                            break

        # 备选：旧版价格选择器
        if not info['price']:
            for selector_id in ['priceblock_ourprice', 'priceblock_dealprice', 'priceblock_saleprice']:
                price_elem = soup.find('span', id=selector_id)
                if price_elem:
                    price_text = price_elem.get_text(strip=True)
                    if price_text and any(c.isdigit() for c in price_text):
                        info['price'] = price_text
                        break

        # 备选：查找 "其他卖家" 价格文本（如 "8 Angebote ab 20,85 €"）
        # 但只在 olp_feature_div 区域内查找
        if not info['price']:
            olp_div = soup.find('div', id='olp_feature_div')
            if olp_div:
                for link in olp_div.find_all('a', href=re.compile(r'/gp/offer-listing/')):
                    text = link.get_text(strip=True)
                    # 匹配欧元价格 (如 "ab 17,69 €")
                    match = re.search(r'ab\s+(\d+[,.]?\d*)\s*€', text, re.IGNORECASE)
                    if match:
                        info['price'] = match.group(1).replace('.', ',') + ' €'
                        break
                    # 匹配其他格式
                    match = re.search(r'(\d+[,.]?\d*)\s*€', text)
                    if match:
                        info['price'] = match.group(1).replace('.', ',') + ' €'
                        break

        # 备选：从 apex_desktop 区域的特定位置获取价格
        if not info['price']:
            apex = soup.find('div', id='apex_desktop')
            if apex:
                # 查找价格但排除轮播图
                for price_span in apex.find_all('span', class_='a-price'):
                    # 确保不在轮播图区域
                    parent_classes = ' '.join(price_span.find_parent('div').get('class', []) if price_span.find_parent('div') else [])
                    if 'p13n' in parent_classes or 'carousel' in parent_classes or 'sims' in parent_classes:
                        continue
                    if 'a-text-price' in price_span.get('class', []):
                        continue
                    price_offscreen = price_span.find('span', class_='a-offscreen')
                    if price_offscreen:
                        info['price'] = price_offscreen.get_text(strip=True)
                        break

        # 如果商品不可用且没有找到价格，不要从全局查找（避免获取轮播图价格）
        # 这种情况下价格将保持为 None

        # 评分
        rating_elem = soup.find('span', class_='a-icon-alt')
        if rating_elem:
            # 确保不是轮播图中的评分
            parent = rating_elem.find_parent('div', id='averageCustomerReviews')
            if parent or rating_elem.find_parent('div', id='detailBullets_feature_div'):
                rating_text = rating_elem.get_text(strip=True)
                match = re.search(r'(\d+[.,]?\d*)', rating_text)
                if match:
                    try:
                        info['rating'] = float(match.group(1).replace(',', '.'))
                    except:
                        pass

        # 如果上面没找到评分，尝试其他方式
        if not info['rating']:
            rating_container = soup.find('div', id='averageCustomerReviews')
            if rating_container:
                rating_elem = rating_container.find('span', class_='a-icon-alt')
                if rating_elem:
                    rating_text = rating_elem.get_text(strip=True)
                    match = re.search(r'(\d+[.,]?\d*)', rating_text)
                    if match:
                        try:
                            info['rating'] = float(match.group(1).replace(',', '.'))
                        except:
                            pass

        # 评论数
        reviews_elem = soup.find('span', id='acrCustomerReviewText')
        if reviews_elem:
            reviews_text = reviews_elem.get_text(strip=True)
            # 移除千位分隔符并提取数字
            nums = re.sub(r'[\s.,\u00a0\u202f]', '', reviews_text)
            nums = re.sub(r'[^\d]', '', nums)
            if nums:
                try:
                    info['reviews_count'] = int(nums)
                except:
                    pass

        # 图片 - 主图
        img_elem = soup.find('img', id='landingImage')
        if not img_elem:
            img_elem = soup.find('img', id='imgBlkFront')
        if img_elem:
            # 优先使用 data-old-hires 或 data-a-dynamic-image
            img_url = img_elem.get('data-old-hires') or img_elem.get('src')
            if img_url:
                info['image_url'] = img_url

    except Exception as e:
        pass  # 静默失败，返回空信息

    return info


def main():
    if len(sys.argv) < 4:
        print(json.dumps({
            "error": "用法: python amazon_crawler.py <keyword> <asin> <country> [max_pages] [proxy]"
        }))
        sys.exit(1)

    keyword = sys.argv[1]
    target_asin = sys.argv[2]
    country = sys.argv[3].upper()
    max_pages = int(sys.argv[4]) if len(sys.argv) > 4 else 3
    proxy = sys.argv[5] if len(sys.argv) > 5 else None

    result = search_keyword(keyword, target_asin, country, max_pages, proxy)
    # Windows 默认使用 GBK 编码，强制使用 UTF-8
    output = json.dumps(result, ensure_ascii=False)
    sys.stdout.buffer.write(output.encode('utf-8'))
    sys.stdout.buffer.write(b'\n')


if __name__ == "__main__":
    main()
