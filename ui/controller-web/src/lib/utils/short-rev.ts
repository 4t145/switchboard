export function shortRev(rev: string): string {
    return rev.length > 7 ? rev.slice(0, 7) : rev;
}