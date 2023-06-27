import { createReadStream } from 'fs';
import { createHash } from 'crypto';

export function calculateHashFromFile(filePath: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const hash = createHash('sha256');
    const stream = createReadStream(filePath);

    stream.on('data', (data) => {
      hash.update(data);
    });

    stream.on('end', () => {
      const fileHash = hash.digest('base64');
      resolve(fileHash);
    });

    stream.on('error', (error) => {
      reject(error);
    });
  });
}

// // Usage example
// const filePath = 'path/to/your/file.ext';
// calculateHashFromFile(filePath)
//   .then((hash) => {
//     console.log('SHA256 Hash:', hash);
//   })
//   .catch((error) => {
//     console.error('Error:', error);
//   });
