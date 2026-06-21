"""
游戏主入口。
提供命令行接口选择运行模式。
"""

import sys
import subprocess
import os


def clear_screen():
    """清屏。"""
    os.system('cls' if os.name == 'nt' else 'clear')


def print_menu():
    """打印主菜单。"""
    print("""
    ===================================
        肉鸽战棋游戏 - 测试平台
    ===================================
    1. 运行自动测试（命令行模拟）
    2. 启动图形界面（GUI）
    3. 运行单元测试（pytest）
    4. 查看API文档
    5. 退出
    """)


def run_auto_test():
    """运行自动测试脚本。"""
    print("正在运行自动测试...")
    try:
        subprocess.run([sys.executable, "test_game.py"], check=True)
    except subprocess.CalledProcessError as e:
        print(f"测试运行失败：{e}")
    input("\n按 Enter 键返回主菜单...")


def launch_gui():
    """启动图形界面。"""
    print("正在启动图形界面...")
    try:
        subprocess.run([sys.executable, "gui.py"], check=True)
    except subprocess.CalledProcessError as e:
        print(f"GUI启动失败：{e}")
        print("请确保已安装 Tkinter。")
    input("\n按 Enter 键返回主菜单...")


def run_unit_tests():
    """运行单元测试（pytest）。"""
    print("正在运行单元测试...")
    try:
        subprocess.run([sys.executable, "-m", "pytest", "-v"], check=True)
    except subprocess.CalledProcessError as e:
        print(f"单元测试失败：{e}")
    input("\n按 Enter 键返回主菜单...")


def view_api_docs():
    """查看API文档。"""
    import webbrowser
    try:
        # 尝试用浏览器打开 Markdown 文件
        webbrowser.open('api_doc_1.md')
    except Exception as e:
        print(f"打开文档失败：{e}")
        print("请直接打开 api_doc_1.md 文件查看。")
    input("\n按 Enter 键返回主菜单...")


def main():
    """主循环。"""
    while True:
        clear_screen()
        print_menu()
        choice = input("请选择 (1-5): ").strip()
        if choice == '1':
            run_auto_test()
        elif choice == '2':
            launch_gui()
        elif choice == '3':
            run_unit_tests()
        elif choice == '4':
            view_api_docs()
        elif choice == '5':
            print("再见！")
            sys.exit(0)
        else:
            print("无效选择，请重新输入。")
            input("按 Enter 键继续...")


if __name__ == "__main__":
    main()