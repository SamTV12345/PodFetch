export interface DiskModel {
    DiskType: string,
    name: string,
    file_system: number[],
    mount_point: string,
    total_space: number,
    available_space: number,
    is_removeable: boolean,
}
