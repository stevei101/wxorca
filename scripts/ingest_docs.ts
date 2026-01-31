#!/usr/bin/env bun
/**
 * Documentation Ingestion Script for WXOrca
 * Fetches IBM WatsonX Orchestrate documentation and stores in SurrealDB
 */

import Surreal from "surrealdb";

interface DocRecord {
  title: string;
  content: string;
  category: string;
  url: string;
  created_at: string;
}

const DOCS_TO_INGEST: { url: string; category: string }[] = [
  // Getting Started
  {
    url: "https://www.ibm.com/docs/en/watsonx/watson-orchestrate/base?topic=getting-started-watsonx-orchestrate",
    category: "getting-started",
  },
  // Apps and Skills
  {
    url: "https://www.ibm.com/docs/en/watsonx/watson-orchestrate/base?topic=catalog-overview-apps-skills",
    category: "skills",
  },
  // Building Skills
  {
    url: "https://www.ibm.com/docs/en/watsonx/watson-orchestrate/base?topic=studio-building-skills-skill-flows",
    category: "skills",
  },
  // Enhancing Skills
  {
    url: "https://www.ibm.com/docs/en/watsonx/watson-orchestrate/current?topic=flows-enhancing-publishing-skills",
    category: "skills",
  },
  // Admin Setup
  {
    url: "https://www.ibm.com/docs/en/software-hub/5.1.x?topic=orchestrate-getting-started",
    category: "admin",
  },
];

async function fetchDocContent(url: string): Promise<{ title: string; content: string } | null> {
  try {
    console.log(`Fetching: ${url}`);
    const response = await fetch(url);
    if (!response.ok) {
      console.error(`Failed to fetch ${url}: ${response.status}`);
      return null;
    }

    const html = await response.text();

    // Extract title from <title> tag
    const titleMatch = html.match(/<title>([^<]+)<\/title>/i);
    const title = titleMatch ? titleMatch[1].replace(" - IBM Documentation", "").trim() : "Untitled";

    // Extract main content - look for article or main content div
    // IBM docs typically use specific content containers
    let content = "";

    // Try to extract from common content selectors
    const contentPatterns = [
      /<article[^>]*>([\s\S]*?)<\/article>/gi,
      /<div[^>]*class="[^"]*content[^"]*"[^>]*>([\s\S]*?)<\/div>/gi,
      /<main[^>]*>([\s\S]*?)<\/main>/gi,
    ];

    for (const pattern of contentPatterns) {
      const match = html.match(pattern);
      if (match && match[0]) {
        content = match[0];
        break;
      }
    }

    // Strip HTML tags and clean up
    content = content
      .replace(/<script[^>]*>[\s\S]*?<\/script>/gi, "")
      .replace(/<style[^>]*>[\s\S]*?<\/style>/gi, "")
      .replace(/<[^>]+>/g, " ")
      .replace(/\s+/g, " ")
      .replace(/&nbsp;/g, " ")
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&quot;/g, '"')
      .trim();

    // Limit content length
    if (content.length > 10000) {
      content = content.substring(0, 10000) + "...";
    }

    return { title, content };
  } catch (error) {
    console.error(`Error fetching ${url}:`, error);
    return null;
  }
}

async function main() {
  const surrealHost = process.env.SURREAL_HOST || "localhost";
  const surrealPort = process.env.SURREAL_PORT || "8000";
  const surrealUser = process.env.SURREAL_USER || "root";
  const surrealPass = process.env.SURREAL_PASS || "root";

  console.log(`Connecting to SurrealDB at ${surrealHost}:${surrealPort}...`);

  const db = new Surreal();

  try {
    await db.connect(`ws://${surrealHost}:${surrealPort}/rpc`);
    await db.signin({ username: surrealUser, password: surrealPass });
    await db.use({ namespace: "wxorca", database: "main" });

    console.log("Connected to SurrealDB");

    // Create table if not exists
    await db.query(`
      DEFINE TABLE IF NOT EXISTS wxo_docs SCHEMAFULL;
      DEFINE FIELD title ON wxo_docs TYPE string;
      DEFINE FIELD content ON wxo_docs TYPE string;
      DEFINE FIELD category ON wxo_docs TYPE string;
      DEFINE FIELD url ON wxo_docs TYPE string;
      DEFINE FIELD created_at ON wxo_docs TYPE datetime DEFAULT time::now();
      DEFINE INDEX idx_category ON wxo_docs FIELDS category;
      DEFINE INDEX idx_url ON wxo_docs FIELDS url UNIQUE;
    `);

    console.log(`\nIngesting ${DOCS_TO_INGEST.length} documentation pages...\n`);

    let successCount = 0;
    for (const doc of DOCS_TO_INGEST) {
      const result = await fetchDocContent(doc.url);
      if (result) {
        try {
          // Upsert based on URL
          await db.query(
            `
            DELETE FROM wxo_docs WHERE url = $url;
            CREATE wxo_docs SET
              title = $title,
              content = $content,
              category = $category,
              url = $url,
              created_at = time::now()
          `,
            {
              title: result.title,
              content: result.content,
              category: doc.category,
              url: doc.url,
            }
          );
          console.log(`✓ Ingested: ${result.title}`);
          successCount++;
        } catch (dbError) {
          console.error(`✗ Failed to store: ${result.title}`, dbError);
        }
      }
    }

    console.log(`\n✓ Successfully ingested ${successCount}/${DOCS_TO_INGEST.length} documents`);

    // Show what's in the database
    const docs = await db.query<DocRecord[][]>("SELECT title, category, url FROM wxo_docs");
    console.log("\nDocuments in database:");
    for (const doc of docs[0] || []) {
      console.log(`  - [${doc.category}] ${doc.title}`);
    }
  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  } finally {
    await db.close();
  }
}

main();
