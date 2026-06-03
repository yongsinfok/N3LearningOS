import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface SessionSummaryProps {
  totalCards: number;
  correctCount: number;
  timeSpentSecs: number;
  onFinish: () => void;
}

export default function SessionSummary({
  totalCards, correctCount, timeSpentSecs, onFinish,
}: SessionSummaryProps) {
  const accuracy = totalCards > 0 ? Math.round((correctCount / totalCards) * 100) : 0;
  const minutes = Math.floor(timeSpentSecs / 60);
  const seconds = timeSpentSecs % 60;

  return (
    <Card className="mx-auto max-w-md">
      <CardHeader>
        <CardTitle className="text-center">复习完成!</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3 text-center">
        <div className="text-4xl">{accuracy >= 80 ? "🎉" : "💪"}</div>
        <p className="text-lg">{accuracy}% 正确率</p>
        <div className="flex justify-center gap-6 text-sm text-muted-foreground">
          <span>📝 {totalCards} 张</span>
          <span>✅ {correctCount} 正确</span>
          <span>⏱️ {minutes}:{seconds.toString().padStart(2, "0")}</span>
        </div>
        <Button onClick={onFinish} className="mt-4 w-full">返回仪表盘</Button>
      </CardContent>
    </Card>
  );
}
