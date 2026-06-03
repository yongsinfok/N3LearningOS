import { useState } from "react";
import { Card, CardContent } from "@/components/ui/card";

interface FlashcardProps {
  front: React.ReactNode;
  back: React.ReactNode;
  onShowAnswer?: () => void;
}

export default function Flashcard({ front, back, onShowAnswer }: FlashcardProps) {
  const [flipped, setFlipped] = useState(false);

  const handleFlip = () => {
    if (!flipped) {
      setFlipped(true);
      onShowAnswer?.();
    }
  };

  return (
    <Card
      className="mx-auto min-h-[280px] w-full max-w-lg cursor-pointer select-none transition-all duration-300 hover:shadow-lg"
      onClick={handleFlip}
    >
      <CardContent className="flex min-h-[280px] items-center justify-center p-8">
        {!flipped ? (
          <div className="text-center">
            <p className="text-4xl font-bold">{front}</p>
            <p className="mt-4 text-sm text-muted-foreground">点击翻转</p>
          </div>
        ) : (
          <div className="text-center">{back}</div>
        )}
      </CardContent>
    </Card>
  );
}
