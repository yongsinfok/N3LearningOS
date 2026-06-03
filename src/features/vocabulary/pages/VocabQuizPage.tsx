import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import ReviewQueue from "@/shared/components/review/ReviewQueue";
import SessionSummary from "@/shared/components/review/SessionSummary";
import QuizSetup from "../components/QuizSetup";
import { useDueFlashcards, useReviewFlashcard } from "../hooks/useVocabulary";

export default function VocabQuizPage() {
  const [phase, setPhase] = useState<"setup" | "quiz" | "done">("setup");
  const [currentIdx, setCurrentIdx] = useState(0);
  const [correctCount, setCorrectCount] = useState(0);
  const [startTime] = useState(Date.now());
  const navigate = useNavigate();

  const { data: cards, isLoading } = useDueFlashcards("vocabulary", 20);
  const reviewCard = useReviewFlashcard();

  const handleStart = (_mode: string, _count: number) => setPhase("quiz");

  const handleAnswer = (correct: boolean) => {
    if (correct) setCorrectCount(c => c + 1);
  };

  const handleFinish = async () => {
    if (cards) {
      for (const [flashcard] of cards.slice(0, currentIdx + 1)) {
        await reviewCard(flashcard.id, 2, 5);
      }
    }
    setPhase("done");
  };

  if (phase === "setup") {
    return (
      <div className="space-y-6">
        <Button variant="ghost" onClick={() => navigate("/vocabulary")}>← 返回</Button>
        <h1 className="text-2xl font-bold">词汇测验</h1>
        {isLoading ? (
          <Skeleton className="h-48 w-full max-w-lg" />
        ) : cards && cards.length > 0 ? (
          <div className="space-y-4">
            <p>待复习: {cards.length} 张卡片</p>
            <QuizSetup onStart={handleStart} />
          </div>
        ) : (
          <div className="py-12 text-center text-muted-foreground">暂无待复习的卡片</div>
        )}
      </div>
    );
  }

  if (phase === "done") {
    const timeSpent = Math.floor((Date.now() - startTime) / 1000);
    return (
      <SessionSummary
        totalCards={currentIdx}
        correctCount={correctCount}
        timeSpentSecs={timeSpent}
        onFinish={() => navigate("/")}
      />
    );
  }

  if (!cards || cards.length === 0) {
    return <div className="py-12 text-center text-muted-foreground">无可用的测验卡片</div>;
  }

  const [flashcard, vocab] = cards[currentIdx] || [null, null];
  if (!flashcard || !vocab) {
    handleFinish();
    return null;
  }

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <Button variant="ghost" onClick={() => setPhase("setup")}>← 退出</Button>
      <ReviewQueue current={currentIdx + 1} total={cards.length} />
      <div className="text-center">
        <p className="text-4xl font-bold">{vocab.word}</p>
      </div>
      <div className="flex justify-center gap-3">
        <Button onClick={() => handleAnswer(true)}>知道了</Button>
        <Button variant="destructive" onClick={() => handleAnswer(false)}>不知道</Button>
      </div>
      <div className="flex justify-between">
        <Button variant="outline" disabled={currentIdx <= 0} onClick={() => setCurrentIdx(i => i - 1)}>上一张</Button>
        {currentIdx < cards.length - 1 ? (
          <Button onClick={() => setCurrentIdx(i => i + 1)}>下一张</Button>
        ) : (
          <Button onClick={handleFinish}>完成复习</Button>
        )}
      </div>
    </div>
  );
}
