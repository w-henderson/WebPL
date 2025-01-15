import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  distDir: "dist",
  output: "export",
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
