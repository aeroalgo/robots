using System;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200007D RID: 125
	[HandlerCategory("vvIchimoku"), HandlerName("KijunSen")]
	public class KijunSen : BaseSen, IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x17000183 RID: 387
		[HandlerParameter(true, "26", Min = "5", Max = "52", Step = "1")]
		public override int Period
		{
			// Token: 0x06000470 RID: 1136 RVA: 0x000174E5 File Offset: 0x000156E5
			get;
			// Token: 0x06000471 RID: 1137 RVA: 0x000174ED File Offset: 0x000156ED
			set;
		}
	}
}
