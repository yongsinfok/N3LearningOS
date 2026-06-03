import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useDashboard } from "../hooks/useDashboard";
import StreakDisplay from "../components/StreakDisplay";
import DueCountCard from "../components/DueCountCard";
import ModuleProgressBar from "../components/ModuleProgress";
import WeeklyActivity from "../components/WeeklyActivity";

export default function DashboardPage() {
  const { data, isLoading, error } = useDashboard();
  const navigate = useNavigate();

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-32 w-full" />
        <Skeleton className="h-20 w-full" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">加载失败</p>
        <p className="text-sm text-muted-foreground">{error.message}</p>
      </div>
    );
  }

  if (!data) {
    return <div className="py-12 text-center text-muted-foreground">暂无数据</div>;
  }

  const hasReview = data.due_count > 0 || data.new_available > 0;

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">继续学习!</h1>
          <StreakDisplay streak={data.streak} />
        </div>
      </div>

      <div className="flex flex-col items-center gap-4">
        <DueCountCard dueCount={data.due_count} newAvailable={data.new_available} />
        <Button
          size="lg"
          className="w-full max-w-sm text-lg"
          onClick={() => navigate("/vocabulary/quiz")}
          disabled={!hasReview}
        >
          {hasReview ? "开始学习! 🎯" : "暂无待复习卡片"}
        </Button>
      </div>

      <ModuleProgressBar modules={data.module_progress} />
      <WeeklyActivity activity={data.weekly_activity} />

      <div className="flex flex-wrap gap-2">
        <Button variant="outline" onClick={() => navigate("/vocabulary")}>📖 词汇</Button>
        <Button variant="outline" onClick={() => navigate("/grammar")}>📝 文法</Button>
        <Button variant="outline" onClick={() => navigate("/kanji")}>🈳 汉字</Button>
        <Button variant="outline" disabled>📰 阅读</Button>
        <Button variant="outline" disabled>🎧 听力</Button>
        <Button variant="outline" disabled>📋 模拟考</Button>
      </div>
    </div>
  );
}
