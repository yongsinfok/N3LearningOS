import { Progress } from "@/components/ui/progress";
import type { ModuleProgress as ModuleProgressType } from "../types";

interface ModuleProgressProps {
  modules: ModuleProgressType[];
}

const labels: Record<string, string> = { vocabulary: "词汇", grammar: "文法", kanji: "汉字" };

export default function ModuleProgressBar({ modules }: ModuleProgressProps) {
  if (!modules.length) return null;
  return (
    <div className="space-y-3">
      <h3 className="font-semibold">学习进度</h3>
      {modules.map((mod) => {
        const pct = mod.total > 0 ? Math.round((mod.completed / mod.total) * 100) : 0;
        return (
          <div key={mod.module}>
            <div className="mb-1 flex justify-between text-sm">
              <span>{labels[mod.module] || mod.module}</span>
              <span className="text-muted-foreground">{mod.completed}/{mod.total} ({pct}%)</span>
            </div>
            <Progress value={pct} className="h-2" />
          </div>
        );
      })}
    </div>
  );
}
