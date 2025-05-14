using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000006 RID: 6
	[HandlerCategory("vvIndicators"), HandlerName("ADXxBB"), InputInfo(0, "Инструмент")]
	public class ADXxBB : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000025 RID: 37 RVA: 0x00002A5D File Offset: 0x00000C5D
		public IList<double> Execute(ISecurity src)
		{
			return ADX.GenADXxBB(src, this.Context, this.ADXperiod, this.BandsPeriod, this.Output);
		}

		// Token: 0x17000009 RID: 9
		[HandlerParameter(true, "14", Min = "10", Max = "20", Step = "1")]
		public int ADXperiod
		{
			// Token: 0x0600001F RID: 31 RVA: 0x00002A2A File Offset: 0x00000C2A
			get;
			// Token: 0x06000020 RID: 32 RVA: 0x00002A32 File Offset: 0x00000C32
			set;
		}

		// Token: 0x1700000A RID: 10
		[HandlerParameter(true, "20", Min = "10", Max = "20", Step = "1")]
		public int BandsPeriod
		{
			// Token: 0x06000021 RID: 33 RVA: 0x00002A3B File Offset: 0x00000C3B
			get;
			// Token: 0x06000022 RID: 34 RVA: 0x00002A43 File Offset: 0x00000C43
			set;
		}

		// Token: 0x1700000C RID: 12
		public IContext Context
		{
			// Token: 0x06000026 RID: 38 RVA: 0x00002A7D File Offset: 0x00000C7D
			get;
			// Token: 0x06000027 RID: 39 RVA: 0x00002A85 File Offset: 0x00000C85
			set;
		}

		// Token: 0x1700000B RID: 11
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1")]
		public int Output
		{
			// Token: 0x06000023 RID: 35 RVA: 0x00002A4C File Offset: 0x00000C4C
			get;
			// Token: 0x06000024 RID: 36 RVA: 0x00002A54 File Offset: 0x00000C54
			set;
		}
	}
}
