export type PodcastAddModel = {
    artworkUrl600: string,
    artistName: string,
    collectionName: string,
}

export type GeneralModel = {
    code: number,
    result: {
        resultCount: number,
        results: PodcastAddModel[]
    }
}
