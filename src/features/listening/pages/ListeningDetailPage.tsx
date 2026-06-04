import { useState, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useListeningDetail } from "../hooks/useListening";
import AudioPlayer from "../components/AudioPlayer";
import TranscriptView from "../components/TranscriptView";
import ListeningQuizSection from "../components/QuizSection";

export default function ListeningDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useListeningDetail(id!);
  const navigate = useNavigate();
  const [currentTime, setCurrentTime] = useState(0);

  const handleSeek = useCallback((time: number) => {
    const audio = document.querySelector("audio");
    if (audio) { audio.currentTime = time; audio.play(); }
  }, []);

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-48 w-full" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">课程不存在</p>
        <Button variant="outline" onClick={() => navigate("/listening")}>返回列表</Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <Button variant="ghost" onClick={() => navigate("/listening")}>← 返回列表</Button>
      <h1 className="text-2xl font-bold">{data.title}</h1>
      <AudioPlayer audioFile={data.audio_file} onTimeUpdate={setCurrentTime} />
      <TranscriptView transcriptJson={data.transcript} currentTime={currentTime} onSeek={handleSeek} />
      <ListeningQuizSection questionsJson={data.questions} />
    </div>
  );
}
