import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "WebPL",
  description: "A Prolog interpreter for the browser",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        {children}
      </body>
    </html>
  );
}
