"""
图形界面 (GUI) 用于测试游戏。
使用 Tkinter 绘制棋盘、棋子，支持点击交互，并显示所有数据。
"""

import tkinter as tk
from tkinter import ttk, scrolledtext
import math
from game import Game
from piece import Piece
from card import SwapPositionCard, PullCard, PushCard, GlobalSpeedCard, SelfDamageCard
from card import SpeedToAttackCard, AttackToSpeedCard, DamageSixMultiplierCard, RebirthCard, AttackCard, MoveCard
from obstacle import Obstacle
from range_matrix import RangeMatrix


class GameGUI:
    """游戏图形界面。"""

    def __init__(self, master):
        self.master = master
        master.title("肉鸽战棋游戏测试界面")
        master.geometry("1200x800")

        # 创建游戏实例
        self.game = Game()
        self.setup_demo_game()

        # 当前选中的棋子
        self.selected_piece = None
        # 当前模式：'select', 'move', 'attack', 'skill'
        self.mode = 'select'

        # 创建界面组件
        self.create_widgets()

        # 绘制初始棋盘
        self.draw_board()

    def setup_demo_game(self):
        """设置演示游戏（与test_game.py相同）。"""
        from ai import SimpleMonsterAI, PassiveAI
        # 创建AI实例
        monster_ai = SimpleMonsterAI(aggressiveness=0.9)
        passive_ai = PassiveAI()

        # 创建三个棋子，分配AI
        self.piece1 = Piece("英雄", x=10, y=10, speed=60.0, attack_power=15.0, move_power=8.0, ai=monster_ai)
        self.piece2 = Piece("怪兽", x=20, y=20, speed=40.0, attack_power=20.0, move_power=5.0, ai=monster_ai)
        self.piece3 = Piece("法师", x=30, y=30, speed=80.0, attack_power=10.0, move_power=10.0, ai=passive_ai)

        # 创建卡牌
        swap_card = SwapPositionCard("换位术", "交换双方位置")
        pull_card = PullCard("引力牵引", "将目标拉近")
        push_card = PushCard("冲击波", "将目标推远")
        speed_card = GlobalSpeedCard("疾风祝福", "全体加速")
        self_damage_card = SelfDamageCard("自残", "减少自身血量")
        speed_to_attack_card = SpeedToAttackCard("迅捷攻击", "速度转攻击", factor=0.2)
        attack_to_speed_card = AttackToSpeedCard("狂暴加速", "攻击转速度", factor=0.1)
        six_mult_card = DamageSixMultiplierCard("六之幸运", "伤害尾数6时翻倍")
        rebirth_card = RebirthCard("重生", "死亡后复活", revive_health=30.0)
        attack_card = AttackCard("火球术", "造成火焰伤害", damage_multiplier=1.5)
        move_card = MoveCard("瞬步", "额外移动", move_bonus=3)

        # 装备卡牌
        self.piece1.equip_card(swap_card)
        self.piece1.equip_card(speed_to_attack_card)
        self.piece2.equip_card(pull_card)
        self.piece2.equip_card(attack_to_speed_card)
        self.piece3.equip_card(rebirth_card)
        self.piece3.equip_card(attack_card)

        # 添加到游戏
        self.game.add_piece(self.piece1)
        self.game.add_piece(self.piece2)
        self.game.add_piece(self.piece3)

        # 添加障碍物
        rock = Obstacle("岩石", blocks_movement=True)
        self.game.add_obstacle(rock, x=15, y=15)
        tree = Obstacle("树木", blocks_movement=True, blocks_vision=True)
        self.game.add_obstacle(tree, x=25, y=25)

    def create_widgets(self):
        """创建所有GUI控件。"""
        # 左侧：棋盘画布与滚动条
        self.canvas_frame = ttk.Frame(self.master)
        self.canvas_frame.grid(row=0, column=0, sticky="nsew", padx=10, pady=10)

        self.canvas = tk.Canvas(self.canvas_frame, width=800, height=600, bg="white")
        self.canvas.grid(row=0, column=0, sticky="nsew")

        # 画布绑定点击事件
        self.canvas.bind("<Button-1>", self.on_canvas_click)

        # 右侧：控制面板与信息
        self.info_frame = ttk.Frame(self.master)
        self.info_frame.grid(row=0, column=1, sticky="nsew", padx=10, pady=10)

        # 游戏信息标签
        self.info_label = ttk.Label(self.info_frame, text="游戏信息", font=("Arial", 14))
        self.info_label.pack(pady=5)

        # 详细信息文本框
        self.info_text = scrolledtext.ScrolledText(self.info_frame, width=40, height=20, wrap=tk.WORD)
        self.info_text.pack(pady=5)

        # 棋子列表
        self.pieces_listbox = tk.Listbox(self.info_frame, height=10)
        self.pieces_listbox.pack(pady=5)
        self.pieces_listbox.bind("<<ListboxSelect>>", self.on_piece_select)

        # 按钮面板
        self.button_frame = ttk.Frame(self.info_frame)
        self.button_frame.pack(pady=10)

        ttk.Button(self.button_frame, text="下一回合", command=self.next_round).grid(row=0, column=0, padx=5)
        ttk.Button(self.button_frame, text="移动模式", command=lambda: self.set_mode('move')).grid(row=0, column=1, padx=5)
        ttk.Button(self.button_frame, text="攻击模式", command=lambda: self.set_mode('attack')).grid(row=1, column=0, padx=5)
        ttk.Button(self.button_frame, text="技能模式", command=lambda: self.set_mode('skill')).grid(row=1, column=1, padx=5)
        ttk.Button(self.button_frame, text="取消选择", command=self.deselect).grid(row=2, column=0, columnspan=2, pady=5)

        # 状态栏
        self.status_label = ttk.Label(self.info_frame, text="就绪", relief=tk.SUNKEN)
        self.status_label.pack(side=tk.BOTTOM, fill=tk.X, pady=5)

        # 配置网格权重
        self.master.grid_rowconfigure(0, weight=1)
        self.master.grid_columnconfigure(0, weight=3)
        self.master.grid_columnconfigure(1, weight=1)

    def draw_board(self):
        """绘制棋盘（格子、障碍物、棋子）。"""
        self.canvas.delete("all")
        scale = 10  # 每个格子10像素
        # 绘制格子
        for x in range(0, 100, 5):  # 每5格画一条线，避免太密集
            for y in range(0, 100, 5):
                x1 = x * scale
                y1 = y * scale
                x2 = (x + 5) * scale
                y2 = (y + 5) * scale
                self.canvas.create_rectangle(x1, y1, x2, y2, outline="lightgray", fill="white")

        # 绘制障碍物
        for x in range(100):
            for y in range(100):
                grid = self.game.board.get_grid(x, y)
                for state in grid.states:
                    if isinstance(state, Obstacle):
                        # 绘制棕色方块
                        self.canvas.create_rectangle(
                            x * scale, y * scale,
                            (x + 1) * scale, (y + 1) * scale,
                            fill="brown", outline="black"
                        )

        # 绘制棋子
        for piece in self.game.pieces:
            color = "blue" if piece.name == "英雄" else "red" if piece.name == "怪兽" else "green"
            x = piece.x * scale + scale // 2
            y = piece.y * scale + scale // 2
            radius = scale // 2 - 2
            self.canvas.create_oval(x - radius, y - radius, x + radius, y + radius, fill=color, outline="black")
            # 棋子名称标签
            self.canvas.create_text(x, y - radius - 5, text=piece.name, font=("Arial", 8))

        # 更新棋子列表框
        self.update_pieces_list()

        # 更新信息文本
        self.update_info_text()

    def update_pieces_list(self):
        """更新棋子列表框。"""
        self.pieces_listbox.delete(0, tk.END)
        for piece in self.game.pieces:
            status = "存活" if piece.is_alive() else "死亡"
            self.pieces_listbox.insert(tk.END, f"{piece.name} ({piece.x},{piece.y}) {status}")

    def update_info_text(self):
        """更新信息文本框。"""
        self.info_text.delete(1.0, tk.END)
        text = ""
        for piece in self.game.pieces:
            text += f"--- {piece.name} ---\n"
            text += f"位置: ({piece.x},{piece.y})\n"
            text += f"生命: {piece.health.value:.1f}\n"
            text += f"速度: {piece.speed.value:.1f}\n"
            text += f"攻击: {piece.attack_power.value:.1f}\n"
            text += f"移动: {piece.move_power.value:.1f}\n"
            text += f"行动点数: {piece.action_points}\n"
            text += f"装备卡牌: {[c.name for c in piece.equipped_cards]}\n"
            text += "\n"
        self.info_text.insert(1.0, text)

    def on_canvas_click(self, event):
        """处理画布点击事件。"""
        scale = 10
        x = event.x // scale
        y = event.y // scale
        if 0 <= x < 100 and 0 <= y < 100:
            # 查找点击的棋子
            clicked_piece = None
            for piece in self.game.pieces:
                if piece.x == x and piece.y == y:
                    clicked_piece = piece
                    break

            if self.mode == 'select':
                # 选择棋子
                self.selected_piece = clicked_piece
                self.status_label.config(text=f"选中: {clicked_piece.name if clicked_piece else '空'}")
                self.highlight_selected()
            elif self.mode == 'move' and self.selected_piece:
                # 移动选中棋子到该位置
                self.selected_piece.move_to(x, y)
                self.status_label.config(text=f"{self.selected_piece.name} 移动到 ({x},{y})")
                self.draw_board()
            elif self.mode == 'attack' and self.selected_piece and clicked_piece and clicked_piece != self.selected_piece:
                # 攻击目标
                damage = self.selected_piece.attack_power.value
                clicked_piece.take_damage(damage)
                self.status_label.config(text=f"{self.selected_piece.name} 攻击 {clicked_piece.name} 造成 {damage} 伤害")
                self.draw_board()
            elif self.mode == 'skill' and self.selected_piece and clicked_piece:
                # 使用第一个技能卡牌
                if self.selected_piece.equipped_cards:
                    card = self.selected_piece.equipped_cards[0]
                    card.apply(self.selected_piece, clicked_piece, self.game.board, pieces=self.game.pieces)
                    self.status_label.config(text=f"{self.selected_piece.name} 使用 {card.name}")
                    self.draw_board()

    def on_piece_select(self, event):
        """当从列表框中选择棋子时。"""
        selection = self.pieces_listbox.curselection()
        if selection:
            index = selection[0]
            if index < len(self.game.pieces):
                self.selected_piece = self.game.pieces[index]
                self.status_label.config(text=f"选中: {self.selected_piece.name}")
                self.highlight_selected()

    def highlight_selected(self):
        """高亮显示选中的棋子。"""
        self.canvas.delete("highlight")
        if self.selected_piece:
            scale = 10
            x = self.selected_piece.x * scale
            y = self.selected_piece.y * scale
            self.canvas.create_rectangle(
                x, y, x + scale, y + scale,
                outline="yellow", width=3, tags="highlight"
            )

    def set_mode(self, mode):
        """设置当前模式。"""
        self.mode = mode
        self.status_label.config(text=f"模式: {mode}")

    def deselect(self):
        """取消选择。"""
        self.selected_piece = None
        self.mode = 'select'
        self.canvas.delete("highlight")
        self.status_label.config(text="已取消选择")

    def next_round(self):
        """执行下一回合。"""
        self.game.play_round()
        self.draw_board()
        self.status_label.config(text=f"第 {self.game.round} 回合结束")


def main():
    root = tk.Tk()
    gui = GameGUI(root)
    root.mainloop()


if __name__ == "__main__":
    main()