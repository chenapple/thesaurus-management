#!/usr/bin/env python3
"""
PDF 转图片脚本
将 PDF 的每一页转换为 PNG 图片，用于后续 OCR 识别

依赖安装:
  pip install pdf2image
  brew install poppler  # macOS
  # 或 apt install poppler-utils  # Linux
"""

import sys
import os
import json
import base64
from pathlib import Path

def check_dependencies():
    """检查依赖是否安装"""
    try:
        from pdf2image import convert_from_path
        return True
    except ImportError:
        return False

def pdf_to_images(pdf_path: str, dpi: int = 200) -> list:
    """
    将 PDF 转换为图片列表

    Args:
        pdf_path: PDF 文件路径
        dpi: 图片分辨率（默认 200，平衡质量和大小）

    Returns:
        图片数据列表，每个元素包含 page_number 和 base64_data
    """
    from pdf2image import convert_from_path
    import io

    # 转换 PDF 为图片
    images = convert_from_path(pdf_path, dpi=dpi)

    result = []
    for i, image in enumerate(images):
        # 转换为 base64
        buffer = io.BytesIO()
        image.save(buffer, format='PNG')
        base64_data = base64.b64encode(buffer.getvalue()).decode('utf-8')

        result.append({
            'page_number': i + 1,
            'mime_type': 'image/png',
            'base64_data': base64_data
        })

    return result

def main():
    if len(sys.argv) < 2:
        print(json.dumps({
            'success': False,
            'error': '请提供 PDF 文件路径'
        }))
        sys.exit(1)

    pdf_path = sys.argv[1]
    dpi = int(sys.argv[2]) if len(sys.argv) > 2 else 200

    # 检查文件是否存在
    if not os.path.exists(pdf_path):
        print(json.dumps({
            'success': False,
            'error': f'文件不存在: {pdf_path}'
        }))
        sys.exit(1)

    # 检查依赖
    if not check_dependencies():
        print(json.dumps({
            'success': False,
            'error': '缺少依赖，请运行: pip install pdf2image && brew install poppler'
        }))
        sys.exit(1)

    try:
        images = pdf_to_images(pdf_path, dpi)
        print(json.dumps({
            'success': True,
            'page_count': len(images),
            'images': images
        }))
    except Exception as e:
        print(json.dumps({
            'success': False,
            'error': str(e)
        }))
        sys.exit(1)

if __name__ == '__main__':
    main()
