import SkeletonLoading from "expo-skeleton-loading";
import {View} from "react-native";

export const LoadingSkeleton = ()=>{
    return  <SkeletonLoading background={"lightgrey"} highlight={"white"}>
        <View style={{marginRight: 10}}>
            <View style={{ flexDirection: 'row', justifyContent: 'space-between' }}>
                <View style={{ width: 100, height: 100, backgroundColor: "#adadad", borderRadius: 10 }} />
            </View>

            <View style={{ marginTop: 10 }}>
                <View style={{ backgroundColor: "#adadad", width: 60, height: 10, marginBottom: 5, borderRadius: 5, marginLeft: 5 }} />
                <View style={{ backgroundColor: "#adadad", width: 80, height: 10, marginBottom: 5, borderRadius: 5, marginLeft: 5 }} />
            </View>
        </View>
    </SkeletonLoading>
}
