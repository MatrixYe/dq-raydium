#!/bin/bash

# 日期格式校验函数（必须为8位数字，格式：YYYYMMDD）
function validate_date() {
  if [[ ! $1 =~ ^[0-9]{8}$ ]]; then
    return 1
  else
    return 0
  fi
}

# 输入开始日期并校验
while true; do
  echo "请输入开始日期（格式：20240725）："
  # shellcheck disable=SC2162
  read start_date
  if [ -z "$start_date" ]; then
    echo "开始日期不能为空。"
  elif ! validate_date "$start_date"; then
    echo "开始日期格式不正确，请使用 YYYYMMDD 格式。"
  else
    break
  fi
done

# 输入结束日期并校验
while true; do
  echo "请输入结束日期（格式：20240816）："
  # shellcheck disable=SC2162
  read end_date
  if [ -z "$end_date" ]; then
    echo "结束日期不能为空。"
  elif ! validate_date "$end_date"; then
    echo "结束日期格式不正确，请使用 YYYYMMDD 格式。"
  else
    break
  fi
done

# 输入MongoDB服务器的IP地址，提供默认值127.0.0.1
# shellcheck disable=SC2162
read -p "请输入MongoDB服务器的IP地址（默认：127.0.0.1）： " mongo_ip
mongo_ip=${mongo_ip:-127.0.0.1}

# 日期转换为ISO 8601格式
start_iso="${start_date:0:4}-${start_date:4:2}-${start_date:6:2}T00:00:00.000Z"
end_iso="${end_date:0:4}-${end_date:4:2}-${end_date:6:2}T00:00:00.000Z"

# 输出文件名
output_csv="raydium_actions_${start_date}_${end_date}.csv"
output_gz="raydium_actions_${start_date}_${end_date}.csv.gz"

# 打印确认信息
echo "确认以下参数是否正确："
echo "时间范围：从 ${start_iso} 到 ${end_iso}"
echo "CSV 文件名称：${output_csv}"
echo "压缩文件名称：${output_gz}"
echo "MongoDB 服务器IP地址：${mongo_ip}"

# 用户确认
# shellcheck disable=SC2162
read -p "输入 Y 继续，输入 N 退出：" confirmation

if [[ "$confirmation" != "Y" && "$confirmation" != "y" ]]; then
  echo "操作已取消。"
  exit 0
fi

# 导出数据
mongoexport --uri="mongodb://root:nopassword@${mongo_ip}:5011/" \
  --authenticationDatabase admin \
  --db=raydium --collection=actions \
  --query="{ \"date_time\": { \"\$gte\": { \"\$date\": \"${start_iso}\" }, \"\$lt\": { \"\$date\": \"${end_iso}\" } } }" \
  --type=csv \
  --fields="date_time,metadata.pair,metadata.signer,signature,token_b,fee,decimals_a,action_type,token_a,slot,amount_b,timestamp,amount_a,decimals_b" \
  --out="${output_csv}"

# 检查导出是否成功
if [ $? -ne 0 ]; then
  echo "MongoDB 数据导出失败。脚本中断。"
  exit 1
fi

# 压缩 CSV 文件并显示进度
pv "${output_csv}" | gzip > "${output_gz}"

# 检查压缩是否成功
if [ $? -ne 0 ]; then
  echo "文件压缩失败。脚本中断。"
  exit 1
fi

# 创建目标文件夹（如果不存在）
mkdir -p ~/.file-share-data

# 移动压缩文件到指定文件夹
mv "${output_gz}" ~/.file-share-data/

# 检查文件移动是否成功
if [ $? -ne 0 ]; then
  echo "文件移动失败。脚本中断。"
  exit 1
fi

echo "数据成功导出、压缩并移动到 ~/.file-share-data 文件夹"