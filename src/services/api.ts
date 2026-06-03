import { invoke } from "@tauri-apps/api/core";

export interface Vocabulary {
  id: string;
  word: string;
  reading: string;
  meaning: string;
  example: string | null;
  tags: string | null;
  source: string;
  created_at: string;
}

export interface Flashcard {
  id: string;
  content_id: string;
  content_type: string;
  due_date: string;
  stability: number;
  difficulty: number;
  elapsed_days: number;
  scheduled_days: number;
  reps: number;
  lapses: number;
  state: number;
  last_review: string | null;
  created_at: string;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface DashboardData {
  streak: number;
  due_count: number;
  new_available: number;
  module_progress: ModuleProgress[];
  weekly_activity: DailyActivity[];
}

export interface ModuleProgress {
  module: string;
  completed: number;
  total: number;
}

export interface DailyActivity {
  date: string;
  cards_reviewed: number;
}

export function getDashboard(): Promise<DashboardData> {
  return invoke("get_dashboard");
}

export function listVocabulary(
  page?: number,
  search?: string,
  pageSize?: number,
): Promise<PaginatedResult<Vocabulary>> {
  return invoke("list_vocabulary", { page, search, page_size: pageSize });
}

export function getVocabularyDetail(id: string): Promise<Vocabulary> {
  return invoke("get_vocabulary_detail", { id });
}

export function getDueFlashcards(
  contentType?: string,
  limit?: number,
): Promise<[Flashcard, Vocabulary][]> {
  return invoke("get_due_flashcards", { content_type: contentType, limit });
}

export function reviewFlashcard(
  cardId: string,
  rating: number,
  durationSecs?: number,
): Promise<string> {
  return invoke("review_flashcard", {
    card_id: cardId,
    rating,
    duration_secs: durationSecs,
  });
}
