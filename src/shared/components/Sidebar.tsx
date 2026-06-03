import { NavLink } from "react-router-dom";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/", label: "仪表盘", icon: "📊" },
  { to: "/vocabulary", label: "词汇", icon: "📖" },
  { to: "/grammar", label: "文法", icon: "📝" },
  { to: "/kanji", label: "汉字", icon: "🈳" },
  { to: "/reading", label: "阅读", icon: "📰" },
  { to: "/listening", label: "听力", icon: "🎧" },
  { to: "/notes", label: "笔记", icon: "📝" },
  { to: "/exams", label: "模拟考", icon: "📋" },
];

export default function Sidebar() {
  return (
    <aside className="flex w-56 flex-col border-r bg-muted/30 p-3">
      <nav className="flex flex-col gap-1">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === "/"}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground",
              )
            }
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
