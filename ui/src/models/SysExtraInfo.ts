import {SysInfo} from "./SysInfo";
import {DiskModel} from "./DiskModel";

export interface SysExtraInfo
{
    system: SysInfo,
    disks: DiskModel[]
    podcast_directory: number
}
