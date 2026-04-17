export interface Filter {
    user_id: number
    title?: string | null
    ascending: boolean
    filter?: string | null
    only_favored: boolean
}
