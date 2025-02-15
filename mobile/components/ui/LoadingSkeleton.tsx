import SkeletonLoading from "expo-skeleton-loading";
import {View} from "react-native";

export const LoadingSkeleton = ()=>{
    return  <SkeletonLoading background={"#adadad"} highlight={"#ffffff"}>
        <View style={{ flexDirection: 'row', justifyContent: 'space-between' }}>
            <View style={{ width: 100, height: 100, backgroundColor: "#adadad", borderRadius: 10, marginRight: 10 }} />
        </View>
    </SkeletonLoading>
}
