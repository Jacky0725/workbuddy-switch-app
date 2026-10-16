const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const {spawn}=require('node:child_process');
const worker=path.resolve(__dirname,'../crates/wb-switch-core/src/modules/growth_acp.cjs');
const fake=String.raw`
const fs=require('node:fs');let buffer='';
function send(value){process.stdout.write(JSON.stringify(value)+'\n');}
process.stdin.on('data',chunk=>{buffer+=chunk;while(buffer.includes('\n')){
 const i=buffer.indexOf('\n'),m=JSON.parse(buffer.slice(0,i));buffer=buffer.slice(i+1);
 if(m.method==='initialize')send({jsonrpc:'2.0',id:m.id,result:{protocolVersion:1,agentCapabilities:{metaPromptSupport:true}}});
 else if(m.method==='session/new')send({jsonrpc:'2.0',id:m.id,result:{sessionId:'fictional'}});
 else if(m.method==='session/prompt'){
  const valid=m.params._meta['codebuddy.ai'].mode==='craft'&&process.argv.includes('--tools=')&&process.env.CODEBUDDY_AUTH_TOKEN==='fictional-token'&&!process.env.CODEBUDDY_CUSTOM_HEADERS;
  fs.appendFileSync(process.env.TEST_ACP_RECORD,JSON.stringify({method:m.method,valid})+'\n');
  if(process.env.TEST_ACP_SCENARIO==='client-operation'){send({jsonrpc:'2.0',id:9,method:'fs/read_text_file',params:{path:'fictional'}});continue;}
  if(process.env.TEST_ACP_SCENARIO==='rejection'){send({jsonrpc:'2.0',id:m.id,error:{code:-32000,message:'fictional rejection'}});continue;}
  send({jsonrpc:'2.0',method:'session/update',params:{update:{sessionUpdate:process.env.TEST_ACP_SCENARIO==='echo'?'user_message_chunk':'agent_message_chunk',content:{type:'text',text:'TEST_OK'}}}});
  send({jsonrpc:'2.0',id:m.id,result:{stopReason:'end_turn'}});
  if(process.env.TEST_ACP_SCENARIO==='deferred-completion')setTimeout(()=>fs.appendFileSync(process.env.TEST_ACP_RECORD,JSON.stringify({method:'background-completion'})+'\n'),2000);
 }
}});
`;
async function invoke(t,scenario,preflight=false){
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'workbuddy-growth-acp-test-'));
 const cli=path.join(directory,'fake.cjs'),record=path.join(directory,'calls.jsonl');fs.writeFileSync(cli,fake);
 const child=spawn(process.execPath,[worker],{windowsHide:true,stdio:['pipe','pipe','pipe'],env:{...process.env,TEST_ACP_SCENARIO:scenario,TEST_ACP_RECORD:record,CODEBUDDY_CUSTOM_HEADERS:'must-not-propagate'}});
 t.after(()=>{child.kill();const resolved=fs.realpathSync(directory);assert.equal(path.dirname(resolved),fs.realpathSync(os.tmpdir()));assert.ok(path.basename(resolved).startsWith('workbuddy-growth-acp-test-'));fs.rmSync(resolved,{recursive:true,force:true});});
 let output='',stderr='';child.stdout.on('data',d=>output+=d);child.stderr.on('data',d=>stderr+=d);
 child.stdin.write(JSON.stringify({executable:process.execPath,cli,home:directory,token:'fictional-token',uid:'fictional-user',preflight})+'\n');
 await new Promise((resolve,reject)=>{const timeout=setTimeout(()=>{child.kill();reject(Error('worker timeout'));},10000);child.on('close',()=>{clearTimeout(timeout);resolve();});child.on('error',reject);});
 assert.equal(stderr,'');assert.ok(!output.includes('fictional-token'));
 return{result:JSON.parse(output),calls:fs.existsSync(record)?fs.readFileSync(record,'utf8').trim().split('\n').map(JSON.parse):[]};
}
test('one protocol prompt produces only a bounded summary',async t=>{const {result,calls}=await invoke(t,'success');assert.equal(result.reply_matches,true);assert.equal(result.finished,true);assert.deepEqual(calls,[{method:'session/prompt',valid:true}]);});
test('preflight never sends a model prompt',async t=>{const{result,calls}=await invoke(t,'success',true);assert.equal(result.initialized,true);assert.equal(result.submitted,false);assert.equal(calls.length,0);});
test('echoed user text is not accepted as a model reply',async t=>{const{result}=await invoke(t,'echo');assert.equal(result.reply_matches,false);});
test('unexpected client operations are rejected without further prompts',async t=>{const{result,calls}=await invoke(t,'client-operation');assert.equal(result.error,'unexpected client operation requested');assert.equal(calls.length,1);});
test('protocol rejection cannot be reported as completion',async t=>{const{result,calls}=await invoke(t,'rejection');assert.equal(result.finished,undefined);assert.equal(calls.length,1);assert.ok(result.error);});
test('official runtime remains alive for its deferred completion work',async t=>{const{result,calls}=await invoke(t,'deferred-completion');assert.equal(result.finished,true);assert.equal(result.reply_matches,true);assert.deepEqual(calls.map(c=>c.method),['session/prompt','background-completion']);});
