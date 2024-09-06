/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export declare function getWord(word: string): Promise<string>
export declare function search(queryStr: string, skip: number, limit: number, maxDis: number, mode: string): Promise<string>
export declare function init(): Promise<void>
export interface SearchEntry {
  key: string
  word: string
  pos: string
  en: Array<string>
  matched: string
}
export declare function editDistance(a: string, b: string): number
