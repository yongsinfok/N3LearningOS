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

// === Grammar ===
export interface Grammar {
  id: string;
  pattern: string;
  meaning: string;
  explanation: string;
  examples: string;
  related: string | null;
  source: string;
  created_at: string;
}

export function listGrammar(page?: number, search?: string, pageSize?: number): Promise<PaginatedResult<Grammar>> {
  return invoke("list_grammar", { page, search, page_size: pageSize });
}

export function getGrammarDetail(id: string): Promise<Grammar> {
  return invoke("get_grammar_detail", { id });
}

// === Kanji ===
export interface Kanji {
  id: string;
  character: string;
  onyomi: string | null;
  kunyomi: string | null;
  meaning: string;
  stroke_svg: string | null;
  example_words: string | null;
  source: string;
  created_at: string;
}

export function listKanji(page?: number, search?: string, pageSize?: number): Promise<PaginatedResult<Kanji>> {
  return invoke("list_kanji", { page, search, page_size: pageSize });
}

export function getKanjiDetail(id: string): Promise<Kanji> {
  return invoke("get_kanji_detail", { id });
}

// === Reading ===
export interface Segment {
  t: string;
  f?: string;
  k: boolean;
}

export interface Question {
  id: string;
  question: string;
  options: string[];
  correct_index: number;
  explanation: string;
}

export interface Reading {
  id: string;
  title: string;
  segments: string;    // JSON string
  questions: string | null;
  source: string;
  level: string;
  is_bookmarked: boolean;
  created_at: string;
}

export function listReading(page?: number, search?: string, pageSize?: number): Promise<PaginatedResult<Reading>> {
  return invoke("list_reading", { page, search, page_size: pageSize });
}

export function getReadingDetail(id: string): Promise<Reading> {
  return invoke("get_reading_detail", { id });
}

export function lookupWord(word: string): Promise<Vocabulary | null> {
  return invoke("lookup_word", { word });
}

// === Listening ===
export interface TranscriptEntry {
  start: number;
  end: number;
  text: string;
}

export interface Listening {
  id: string;
  title: string;
  audio_file: string;
  transcript: string;    // JSON
  questions: string | null;
  source: string;
  level: string;
  is_bookmarked: boolean;
  created_at: string;
}

export function listListening(page?: number, search?: string, pageSize?: number): Promise<PaginatedResult<Listening>> {
  return invoke("list_listening", { page, search, page_size: pageSize });
}

export function getListeningDetail(id: string): Promise<Listening> {
  return invoke("get_listening_detail", { id });
}

export function getAudioData(audioFile: string): Promise<number[]> {
  return invoke("get_audio_data", { audio_file: audioFile });
}

// === Search ===
export interface SearchResult {
  id: string;
  content_type: "vocabulary" | "grammar" | "kanji";
  title: string;
  subtitle: string;
  match_field: string;
}

export function globalSearch(query: string): Promise<SearchResult[]> {
  return invoke("global_search", { query });
}

// === Notes ===
export interface Note {
  id: string;
  content_id: string;
  content_type: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export function listNotes(contentId?: string, contentType?: string): Promise<Note[]> {
  return invoke("list_notes", { content_id: contentId, content_type: contentType });
}

export function saveNote(contentId: string, contentType: string, content: string): Promise<string> {
  return invoke("save_note", { content_id: contentId, content_type: contentType, content });
}

export function deleteNote(id: string): Promise<void> {
  return invoke("delete_note", { id });
}

// === Import ===
export interface ImportFileResult {
  imported: number;
  overwritten: number;
  errors: string[];
  total_in_file: number;
}

export function importVocabularyFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_vocabulary_from_file", { filePath, mode });
}

export function importGrammarFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_grammar_from_file", { filePath, mode });
}

export function importKanjiFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_kanji_from_file", { filePath, mode });
}

export function importReadingFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_reading_from_file", { filePath, mode });
}

export function importListeningFromFile(
  filePath: string,
  mode: "Append" | "Overwrite",
): Promise<ImportFileResult> {
  return invoke("import_listening_from_file", { filePath, mode });
}
