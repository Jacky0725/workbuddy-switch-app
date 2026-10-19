// Finite official-runtime adapter. No direct activity-report requests.
const {spawn}=require('node:child_process');
const {mkdirSync}=require('node:fs');
let input='',started=false,child,done=false,timer,completionTimer;
function finish(value){
 if(done)return;done=true;clearTimeout(timer);clearTimeout(completionTimer);
 if(child){child.stdin.end();child.kill();}
 process.stdout.write(JSON.stringify(value)+'\n',()=>process.exit(0));
}
process.stdin.setEncoding('utf8');
process.stdin.on('end',()=>{if(!done)finish({error:'parent disconnected'});});
process.stdin.on('data',chunk=>{
 if(started)return;
 input+=chunk;if(input.length>65536){finish({error:'input bound exceeded'});return;}
 if(!input.includes('\n'))return;
 started=true;
 let config;try{config=JSON.parse(input.slice(0,input.indexOf('\n')));}catch{finish({error:'invalid input'});return;}
 input='';
 if(!config.executable||!config.cli||!config.home||!config.token||!config.uid){finish({error:'missing runtime input'});return;}
 mkdirSync(config.home,{recursive:true});
 // Do not inherit alternate providers, custom auth headers, plugins or user hooks.
 const env=Object.fromEntries(Object.entries(process.env).filter(([k])=>!/^(CODEBUDDY_|ACC_|CLIENT_INFO_|WORKBUDDY_|NODE_OPTIONS$|NODE_PATH$)/i.test(k)));
 Object.assign(env,{ELECTRON_RUN_AS_NODE:'1',CODEBUDDY_FORCE_LITE_WB_BUNDLE:'1',
  CODEBUDDY_DISABLE_PRODUCT_CACHE:'1',CODEBUDDY_CONFIG_DIR:config.home,WORKBUDDY_CONFIG_DIR:config.home,
  CODEBUDDY_AUTH_TOKEN:config.token,ACC_USER_ID:config.uid,DISABLE_AUTOUPDATER:'1',
  CODEBUDDY_DISABLE_CRON:'1',CODEBUDDY_DISABLE_AUTO_MEMORY:'1',CODEBUDDY_DISABLE_EXTENDED_PLUGIN_HOOKS:'1',
  CODEBUDDY_DISABLE_FORK_SUBAGENT:'1'});
 child=spawn(config.executable,[config.cli,'--serve','--acp','--host','127.0.0.1','--auth','password','--port','0',
  '--tools=','--max-turns','1','--no-session-persistence','--setting-sources=',
  '--strict-mcp-config','--mcp-config','{"mcpServers":{}}','--model','glm-5.0-turbo',
  '--system-prompt','Reply only with TEST_OK. Do not read files, access the web or call tools.'],
 {cwd:config.home,env,windowsHide:true,stdio:['pipe','pipe','pipe']});
 config.token='';config.uid='';
 let buffer='',bytes=0,reply='',submitted=false;
 function send(id,method,params){child.stdin.write(JSON.stringify({jsonrpc:'2.0',id,method,params})+'\n');}
 timer=setTimeout(()=>finish({submitted,error:'runtime timeout; no replay'}),60000);
 child.on('error',()=>finish({submitted,error:'official runtime unavailable'}));
 child.on('close',()=>{if(!done)finish({submitted,error:'official runtime closed'});});
 child.stdin.on('error',()=>finish({submitted,error:'official runtime input closed'}));
 child.stderr.on('data',()=>{});
 child.stdout.setEncoding('utf8');
 child.stdout.on('data',chunk=>{
  bytes+=Buffer.byteLength(chunk);if(bytes>2*1024*1024){finish({submitted,error:'runtime response bound exceeded'});return;}
  buffer+=chunk.toString();
  while(buffer.includes('\n')){
   const index=buffer.indexOf('\n'),line=buffer.slice(0,index);buffer=buffer.slice(index+1);
   let message;try{message=JSON.parse(line);}catch{continue;}
   if(message.method==='session/update'){
    const u=message.params?.update;
    if(u?.sessionUpdate==='agent_message_chunk'&&u.content?.type==='text')reply+=u.content.text;
    if(reply.length>8192){finish({submitted,error:'reply bound exceeded'});return;}
    continue;
   }
   if(message.method&&message.id!==undefined){
    child.stdin.write(JSON.stringify({jsonrpc:'2.0',id:message.id,error:{code:-32000,message:'Client operations disabled'}})+'\n');
    finish({submitted,error:'unexpected client operation requested'});return;
   }
   if(![1,2,4].includes(message.id))continue;
   if(message.error){finish({submitted,error:'official protocol rejected request'});return;}
   if(message.id===1&&message.result){
    if(message.result.protocolVersion!==1||message.result.agentCapabilities?.metaPromptSupport!==true){finish({submitted,error:'unsupported official protocol'});return;}
    send(2,'session/new',{cwd:config.home,mcpServers:[]});
   }else if(message.id===2&&typeof message.result?.sessionId==='string'){
    if(config.preflight===true){finish({initialized:true,submitted:false});return;}
    submitted=true;
    send(4,'session/prompt',{sessionId:message.result.sessionId,
     prompt:[{type:'text',text:'请只回复 TEST_OK。不访问网页，不读取或修改文件，不调用工具。'}],
     _meta:{'codebuddy.ai':{mode:'craft'}}});
   }else if(message.id===4&&message.result){
    const result={submitted,finished:message.result.stopReason==='end_turn',reply_matches:reply.trim()==='TEST_OK'};
    // The verified official bundle batches normal events after 2 seconds. Keep
    // its process alive for that batch and a bounded network margin; only the
    // subsequent official task GET can establish that progress was counted.
    if(!completionTimer)completionTimer=setTimeout(()=>finish(result),5000);
    return;
   }
  }
 });
 send(1,'initialize',{protocolVersion:1,clientInfo:{name:'WorkBuddyManager',version:'0.2.0-alpha.9'},
  clientCapabilities:{fs:{readTextFile:false,writeTextFile:false},terminal:false}});
});
