import requests
import json
from sseclient import SSEClient

class GitAiMcpClient:
    def __init__(self, base_url='http://127.0.0.1:8080'):
        self.base_url = base_url

    def connect_sse(self):
        """连接SSE事件流"""
        messages = SSEClient(f'{self.base_url}/events')
        for msg in messages:
            print(f'收到事件: {msg.data}')

    def get_tools(self):
        """获取工具列表"""
        response = requests.get(f'{self.base_url}/tools/list')
        return response.json()

    def call_tool(self, name, args=None):
        """调用GitAI工具"""
        payload = {'name': name, 'arguments': args or {}}
        response = requests.post(
            f'{self.base_url}/tools/call',
            headers={'Content-Type': 'application/json'},
            data=json.dumps(payload)
        )
        return response.json()

    def commit(self, **options):
        """AI提交"""
        return self.call_tool('gitai_commit', options)

    def review(self, **options):
        """代码评审"""
        return self.call_tool('gitai_review', options)

    def scan(self, **options):
        """代码扫描"""
        return self.call_tool('gitai_scan', options)

# 使用示例
client = GitAiMcpClient()
tools = client.get_tools()
print('可用工具:', tools)

# 执行AI提交
result = client.commit(auto_stage=True, tree_sitter=True)
print('提交结果:', result)
