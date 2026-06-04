import { useState, useMemo } from "react";
import { Question } from "@/services/api";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";

interface Props {
  questionsJson: string | null;
}

export default function ListeningQuizSection({ questionsJson }: Props) {
  const questions: Question[] = useMemo(() => {
    if (!questionsJson) return [];
    try { return JSON.parse(questionsJson); } catch { return []; }
  }, [questionsJson]);

  if (questions.length === 0) return null;

  return (
    <div className="mt-8 space-y-4">
      <Separator />
      <h2 className="text-xl font-bold">聴解問題</h2>
      {questions.map((q) => (
        <QuestionCard key={q.id} question={q} />
      ))}
    </div>
  );
}

function QuestionCard({ question }: { question: Question }) {
  const [selected, setSelected] = useState<number | null>(null);
  const answered = selected !== null;
  const isCorrect = selected === question.correct_index;

  return (
    <div className="rounded-lg border p-4">
      <p className="mb-3 font-medium">{question.question}</p>
      <div className="space-y-2">
        {question.options.map((opt, oi) => {
          let variant: "outline" | "default" | "destructive" = "outline";
          if (answered) {
            if (oi === question.correct_index) variant = "default";
            else if (oi === selected) variant = "destructive";
          }
          return (
            <Button key={oi} variant={variant} className="w-full justify-start text-left" disabled={answered} onClick={() => setSelected(oi)}>
              {opt}
            </Button>
          );
        })}
      </div>
      {answered && (
        <div className={`mt-3 rounded-md p-3 text-sm ${
          isCorrect
            ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300"
            : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300"
        }`}>
          <p className="font-medium">{isCorrect ? "✓ 正解！" : "✗ 不正解"}</p>
          <p className="mt-1">{question.explanation}</p>
        </div>
      )}
    </div>
  );
}
