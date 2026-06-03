import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface QuizSetupProps { onStart: (mode: string, count: number) => void; }

const modes = [
  { id: "jp-en", label: "日 → 英", desc: "看日语选英语" },
  { id: "en-jp", label: "英 → 日", desc: "看英语选日语" },
  { id: "reading", label: "读音选择", desc: "看单词选读音" },
];
const counts = [10, 20, 50];

export default function QuizSetup({ onStart }: QuizSetupProps) {
  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold">选择测验模式</h2>
      {modes.map((mode) => (
        <Card key={mode.id} className="cursor-pointer hover:bg-muted/50">
          <CardHeader><CardTitle className="text-base">{mode.label}</CardTitle></CardHeader>
          <CardContent><p className="text-sm text-muted-foreground">{mode.desc}</p></CardContent>
        </Card>
      ))}
      <h3 className="font-semibold">题目数量</h3>
      <div className="flex gap-2">
        {counts.map((n) => (
          <Button key={n} variant="outline" onClick={() => onStart("jp-en", n)}>{n} 题</Button>
        ))}
      </div>
    </div>
  );
}
