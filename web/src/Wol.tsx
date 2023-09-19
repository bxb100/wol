import {ConfigProvider, App, theme, FloatButton} from 'antd';
import zhCN from 'antd/locale/zh_CN';
import useTheme, {ITheme} from './hooks/useTheme';
import Root from './components/Root';
import {OneToOneOutlined} from "@ant-design/icons";
import {useInterface, useUpdateInterface} from "@/hooks/useDevices";

export default function Wol() {
  const [themeValue] = useTheme();
  const algorithm = themeValue === ITheme.Dark ? theme.darkAlgorithm : theme.defaultAlgorithm;

  const {data, isLoading} = useInterface();
  const {trigger} = useUpdateInterface({
    onSuccess: () => {
      console.log("success")
    }
  });

  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        algorithm,
      }}
    >
      <App>
        <Root/>
        {
          isLoading ? null : <FloatButton.Group
            trigger="hover"
            style={{right: 94}}
            shape="square"
            icon={<OneToOneOutlined/>}
          >
            {data?.map((item) => (
              <FloatButton description={item.name} key={item.mac} style={{width: "fit-content"}}
                           tooltip={<div><p>{item.name}</p><p>{item.ips}</p><p>{item.mac}</p></div>}
                           onClick={() => trigger(item.name)}
                           shape="square"
                           type={item.chosen ? "primary" : "default"}/>
            ))}
          </FloatButton.Group>
        }
      </App>
    </ConfigProvider>
  );
}
