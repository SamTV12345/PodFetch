export interface Filter {
    user_id: string
    title?: string | null
    ascending: boolean
    filter?: string | null
    only_favored: boolean
}
