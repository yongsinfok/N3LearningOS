import { createBrowserRouter, Navigate } from "react-router-dom";
import Layout from "@/shared/components/Layout";
import DashboardPage from "@/features/dashboard/pages/DashboardPage";
import VocabListPage from "@/features/vocabulary/pages/VocabListPage";
import VocabDetailPage from "@/features/vocabulary/pages/VocabDetailPage";
import VocabQuizPage from "@/features/vocabulary/pages/VocabQuizPage";
import GrammarListPage from "@/features/grammar/pages/GrammarListPage";
import GrammarDetailPage from "@/features/grammar/pages/GrammarDetailPage";
import KanjiListPage from "@/features/kanji/pages/KanjiListPage";
import KanjiDetailPage from "@/features/kanji/pages/KanjiDetailPage";
import ReadingListPage from "@/features/reading/pages/ReadingListPage";
import ReadingDetailPage from "@/features/reading/pages/ReadingDetailPage";
import ListeningListPage from "@/features/listening/pages/ListeningListPage";
import ListeningDetailPage from "@/features/listening/pages/ListeningDetailPage";
import NotesPage from "@/features/notes/pages/NotesPage";

export const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: "vocabulary", element: <VocabListPage /> },
      { path: "vocabulary/:id", element: <VocabDetailPage /> },
      { path: "vocabulary/quiz", element: <VocabQuizPage /> },
      { path: "grammar", element: <GrammarListPage /> },
      { path: "grammar/:id", element: <GrammarDetailPage /> },
      { path: "kanji", element: <KanjiListPage /> },
      { path: "kanji/:id", element: <KanjiDetailPage /> },
      { path: "reading", element: <ReadingListPage /> },
      { path: "reading/:id", element: <ReadingDetailPage /> },
      { path: "listening", element: <ListeningListPage /> },
      { path: "listening/:id", element: <ListeningDetailPage /> },
      { path: "notes", element: <NotesPage /> },
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);
