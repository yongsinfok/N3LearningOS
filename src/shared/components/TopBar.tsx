import { Button } from "@/components/ui/button";

interface TopBarProps {
  onToggleTheme: () => void;
  theme: "light" | "dark";
}

export default function TopBar({ onToggleTheme, theme }: TopBarProps) {
  return (
    <header className="flex items-center justify-between border-b px-6 py-3">
      <a href="/" className="flex items-center gap-2">
        <span className="text-xl font-bold">N3学習</span>
      </a>
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" onClick={onToggleTheme} aria-label="Toggle theme">
          {theme === "dark" ? "☀️" : "🌙"}
        </Button>
      </div>
    </header>
  );
}
