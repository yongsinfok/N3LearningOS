import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Question } from "@/services/api";

interface Props {
  question: Question;
  index: number;
}

export default function QuizQuestion({ question, index }: Props) {
  const [selected, setSelected] = useState<number | null>(null);
  const answered = selected !== null;
  const isCorrect = selected === question.correct_index;

  return (
    <div className="rounded-lg border p-4">
      <p className="mb-3 font-medium">
        {index + 1}. {question.question}
      </p>
      <div className="space-y-2">
        {question.options.map((opt, oi) => {
          let variant: "outline" | "default" | "secondary" | "destructive" = "outline";
          if (answered) {
            if (oi === question.correct_index) variant = "default";
            else if (oi === selected) variant = "destructive";
          }
          return (
            <Button
              key={oi}
              variant={variant}
              className="w-full justify-start text-left"
              disabled={answered}
              onClick={() => setSelected(oi)}
            >
              {opt}
            </Button>
          );
        })}
      </div>
      {answered && (
        <div className={`mt-3 rounded-md p-3 text-sm ${isCorrect ? "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300" : "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300"}`}>
          <p className="font-medium">{isCorrect ? "✓ 正解！" : "✗ 不正解"}</p>
          <p className="mt-1">{question.explanation}</p>
        </div>
      )}
    </div>
  );
}
