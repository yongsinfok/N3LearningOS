import { Button } from "@/components/ui/button";

interface ReviewRatingProps {
  onRate: (rating: number) => void;
  disabled?: boolean;
}

const ratings = [
  { value: 0, label: "Again", color: "bg-red-500 hover:bg-red-600" },
  { value: 1, label: "Hard", color: "bg-orange-500 hover:bg-orange-600" },
  { value: 2, label: "Good", color: "bg-green-500 hover:bg-green-600" },
  { value: 3, label: "Easy", color: "bg-blue-500 hover:bg-blue-600" },
];

export default function ReviewRating({ onRate, disabled }: ReviewRatingProps) {
  return (
    <div className="flex justify-center gap-3">
      {ratings.map((r) => (
        <Button
          key={r.value}
          onClick={() => onRate(r.value)}
          disabled={disabled}
          className={`${r.color} min-w-[80px] text-white`}
        >
          {r.label}
        </Button>
      ))}
    </div>
  );
}
