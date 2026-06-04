import { useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useReadingDetail } from "../hooks/useReading";
import ReadingContent from "../components/ReadingContent";
import FuriganaToggle from "../components/FuriganaToggle";
import WordPopover from "../components/WordPopover";
import QuizSection from "../components/QuizSection";

export default function ReadingDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useReadingDetail(id!);
  const navigate = useNavigate();
  const [showFurigana, setShowFurigana] = useState(true);
  const [popover, setPopover] = useState<{ word: string; x: number; y: number } | null>(null);

  const handleWordClick = useCallback((word: string, e: React.MouseEvent) => {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    setPopover({ word, x: rect.left, y: rect.bottom });
  }, []);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-6 w-48" />
        {Array.from({ length: 6 }).map((_, i) => (
          <Skeleton key={i} className="h-4 w-full" />
        ))}
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">文章不存在</p>
        <Button variant="outline" onClick={() => navigate("/reading")}>返回列表</Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex items-center justify-between">
        <Button variant="ghost" onClick={() => navigate("/reading")}>← 返回列表</Button>
        <FuriganaToggle
          show={showFurigana}
          onToggle={() => setShowFurigana((v) => !v)}
        />
      </div>

      <h1 className="text-2xl font-bold">{data.title}</h1>

      <ReadingContent
        segmentsJson={data.segments}
        showFurigana={showFurigana}
        onWordClick={handleWordClick}
      />

      {popover && (
        <WordPopover
          word={popover.word}
          position={{ x: popover.x, y: popover.y }}
          onClose={() => setPopover(null)}
        />
      )}

      <QuizSection questionsJson={data.questions} />
    </div>
  );
}
