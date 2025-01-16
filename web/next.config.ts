import type { NextConfig } from "next";

import { execSync } from "child_process";

const commitHash = execSync('git log --pretty=format:"%h" -n1')
  .toString()
  .trim();

const nextConfig: NextConfig = {
  distDir: "dist",
  output: "export",
  env: {
    NEXT_PUBLIC_GIT_COMMIT_HASH: commitHash,
  },
  webpack: (config, { }) => {
    return {
      ...config,
      resolve: {
        ...config.resolve,
        fallback: {
          ...config.resolve?.fallback,
          fs: false,
          crypto: false,
          path: false,
          child_process: false,
        },
      }
    };
  }
};

export default nextConfig;
