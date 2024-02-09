using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000005 RID: 5
	[HandlerCategory("vvIndicators"), HandlerName("ADX WA")]
	public class ADX_WA : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600001B RID: 27 RVA: 0x000029F7 File Offset: 0x00000BF7
		public IList<double> Execute(ISecurity src)
		{
			return ADX.GenADX_WA(src, this.Context, this.ADXperiod, this.Output);
		}

		// Token: 0x17000006 RID: 6
		[HandlerParameter(true, "13", Min = "0", Max = "20", Step = "1")]
		public int ADXperiod
		{
			// Token: 0x06000017 RID: 23 RVA: 0x000029D5 File Offset: 0x00000BD5
			get;
			// Token: 0x06000018 RID: 24 RVA: 0x000029DD File Offset: 0x00000BDD
			set;
		}

		// Token: 0x17000008 RID: 8
		public IContext Context
		{
			// Token: 0x0600001C RID: 28 RVA: 0x00002A11 File Offset: 0x00000C11
			get;
			// Token: 0x0600001D RID: 29 RVA: 0x00002A19 File Offset: 0x00000C19
			set;
		}

		// Token: 0x17000007 RID: 7
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1", Name = "Output:\n0/1/2")]
		public int Output
		{
			// Token: 0x06000019 RID: 25 RVA: 0x000029E6 File Offset: 0x00000BE6
			get;
			// Token: 0x0600001A RID: 26 RVA: 0x000029EE File Offset: 0x00000BEE
			set;
		}
	}
}
