import { expect,it } from "vitest";
import { redact } from "./redact";
it("错误说明保留业务含义并去掉凭证",()=>{
 const value=redact('网络连接失败 {"access_token":"fiction-secret"} ?refreshToken=fiction-refresh Bearer fiction-bearer');
 expect(value).toContain("网络连接失败");
 for(const secret of ["fiction-secret","fiction-refresh","fiction-bearer"])expect(value).not.toContain(secret);
});
it("去掉 Authorization 请求头和带空格的引号凭证",()=>{
 const value=redact(`Authorization: Bearer fiction-header {"password":"fiction secret with spaces"} secret='another spaced value'`);
 for(const secret of ["fiction-header","fiction secret","another spaced"])expect(value).not.toContain(secret);
});
