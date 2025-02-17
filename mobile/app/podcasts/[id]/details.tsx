import {Image, Text, View} from "react-native";
import {SafeAreaView} from "react-native-safe-area-context";
import {useLocalSearchParams} from "expo-router";
import {$api} from "@/client";
import MaterialIcons from "@expo/vector-icons/MaterialIcons";
import Heading2 from "@/components/text/Heading2";

export default function () {
    const { id } = useLocalSearchParams();
    const {data, isLoading, error} = $api.useQuery('get', '/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: id as string
            }
        }
    })

    console.log(data, error)


    return <SafeAreaView>
        <MaterialIcons name="" />
        {
            !isLoading && data? <>
                <Image src={data.image_url}   style={{
                    width: 200,
                    height: 200,
                    borderRadius: 20,
                    marginLeft: 'auto',
                    marginRight: 'auto',
                    marginTop: 50,
                    shadowColor: '#000',
                    shadowOffset: { width: 0, height: 2 },
                    shadowOpacity: 0.8,
                    shadowRadius: 10,
                    filter: 'drop-shadow(0px 20px 20px rgba(0, 0, 0, 0.25))',
                    elevation: 5, // For Android shadow
                }}
                />
                <Heading2 styles={{marginRight: 'auto', marginLeft: 'auto', width: '95%', marginTop: 10}}>{data.name}</Heading2>
            </>: <>
            </>
        }
    </SafeAreaView>
}
