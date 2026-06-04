import { useMemo } from "react";
import { Question } from "@/services/api";
import QuizQuestion from "./QuizQuestion";
import { Separator } from "@/components/ui/separator";

interface Props {
  questionsJson: string | null;
}

export default function QuizSection({ questionsJson }: Props) {
  const questions: Question[] = useMemo(() => {
    if (!questionsJson) return [];
    try { return JSON.parse(questionsJson); } catch { return []; }
  }, [questionsJson]);

  if (questions.length === 0) return null;

  return (
    <div className="mt-8 space-y-4">
      <Separator />
      <h2 className="text-xl font-bold">読解問題</h2>
      {questions.map((q, i) => (
        <QuizQuestion key={q.id} question={q} index={i} />
      ))}
    </div>
  );
}
