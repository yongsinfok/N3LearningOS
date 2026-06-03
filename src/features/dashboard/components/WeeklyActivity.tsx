import type { DailyActivity } from "../types";

interface WeeklyActivityProps {
  activity: DailyActivity[];
}

export default function WeeklyActivity({ activity }: WeeklyActivityProps) {
  if (!activity.length) {
    return <div className="text-sm text-muted-foreground">本周还没有学习记录</div>;
  }
  return (
    <div>
      <h3 className="mb-2 font-semibold">本周活跃</h3>
      <div className="flex gap-1">
        {activity.map((day, i) => {
          const height = Math.min(day.cards_reviewed * 8, 120) || 4;
          return (
            <div key={day.date} className="flex flex-1 flex-col items-center gap-1">
              <div className="flex flex-1 items-end">
                <div className="w-4 rounded bg-primary transition-all" style={{ height }} />
              </div>
              <span className="text-xs text-muted-foreground">{["一","二","三","四","五","六","日"][i] || ""}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
