import { Outlet } from "react-router-dom";
import TopBar from "./TopBar";
import Sidebar from "./Sidebar";
import { useTheme } from "@/hooks/useTheme";

export default function Layout() {
  const { theme, toggleTheme } = useTheme();

  return (
    <div className="flex h-screen flex-col">
      <TopBar onToggleTheme={toggleTheme} theme={theme} />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-y-auto p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
