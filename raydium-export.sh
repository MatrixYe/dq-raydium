#!/bin/bash

# 日期格式校验函数（必须为8位数字，格式：YYYYMMDD）
validate_date() {
  [[ $1 =~ ^[0-9]{8}$ ]]
}

# 错误处理函数
handle_error() {
  echo "$1 脚本中断。"
  exit 1
}

# 输入并校验开始日期
while true; do
  read -p "请输入开始日期（格式：yyyyMMdd 例如 20240725）： " start_date
  if [[ -z "$start_date" ]]; then
    echo "开始日期不能为空。"
  elif ! validate_date "$start_date"; then
    echo "开始日期格式不正确，请使用 YYYYMMDD 格式。"
  else
    break
  fi
done

# 输入并校验结束日期
while true; do
  read -p "请输入结束日期（格式：yyyyMMdd 例如 20240816）： " end_date
  if [[ -z "$end_date" ]]; then
    echo "结束日期不能为空。"
  elif ! validate_date "$end_date"; then
    echo "结束日期格式不正确，请使用 YYYYMMDD 格式。"
  else
    break
  fi
done

# 输入MongoDB服务器的IP地址，提供默认值127.0.0.1
read -p "请输入MongoDB服务器的IP地址（默认：127.0.0.1）： " mongo_ip
mongo_ip=${mongo_ip:-127.0.0.1}

# 日期转换为ISO 8601格式
start_iso="${start_date:0:4}-${start_date:4:2}-${start_date:6:2}T00:00:00.000Z"
end_iso="${end_date:0:4}-${end_date:4:2}-${end_date:6:2}T00:00:00.000Z"

# 输出文件名
output_csv="raydium_actions_${start_date}_${end_date}.csv"
compressed_output="raydium_actions_${start_date}_${end_date}.csv.gz"

# 打印确认信息
echo "--- --- --- ---"
echo " 确认以下参数是否正确："
echo " 时间范围：从 ${start_iso} 到 ${end_iso}"
echo " CSV 文件名称：${output_csv}"
echo " 压缩文件名称：${compressed_output}"
echo " MongoDB 服务器IP地址：${mongo_ip}"
echo "--- --- --- ---"

# 用户确认
read -p "输入 Y 继续，输入 N 退出：" confirmation

if [[ "$confirmation" != "Y" && "$confirmation" != "y" ]]; then
  echo "操作已取消。"
  exit 0
fi

# 导出数据到 CSV 文件
mongoexport --uri="mongodb://root:nopassword@${mongo_ip}:5011/" \
  --authenticationDatabase admin \
  --db=raydium --collection=actions \
  --query="{ \"date_time\": { \"\$gte\": { \"\$date\": \"${start_iso}\" }, \"\$lt\": { \"\$date\": \"${end_iso}\" } } }" \
  --type=csv \
  --fields="date_time,metadata.pair,metadata.signer,signature,token_b,fee,decimals_a,action_type,token_a,slot,amount_b,timestamp,amount_a,decimals_b" \
  --out="${output_csv}" || handle_error "MongoDB 数据导出失败"

# 检查并压缩 CSV 文件
if command -v pv &> /dev/null; then
  pv "${output_csv}" | gzip > "${compressed_output}"
else
  echo "警告：pv 命令未找到，使用标准 gzip 压缩"
  gzip < "${output_csv}" > "${compressed_output}"
fi

# 检查压缩是否成功
[[ $? -eq 0 ]] || handle_error "文件压缩失败"

# 创建目标文件夹（如果不存在）
mkdir -p ~/.file-share-data || handle_error "创建目标文件夹失败"

# 移动压缩文件到指定文件夹
mv "${compressed_output}" ~/.file-share-data/ || handle_error "文件移动失败"

# 删除原始 CSV 文件
rm -f "${output_csv}" || handle_error "原始文件删除失败"

echo "数据成功导出、压缩并移动到 ~/.file-share-data 文件夹"