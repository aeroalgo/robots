using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B9 RID: 185
	[HandlerCategory("vvTrade"), HandlerName("Больше ли лимита")]
	public class CheckMinLimit : IDoubleCompaper1Handler, IOneSourceHandler, IBooleanReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060006A2 RID: 1698 RVA: 0x0001E2A8 File Offset: 0x0001C4A8
		public IList<bool> Execute(IList<double> src)
		{
			List<bool> list = new List<bool>(src.Count);
			for (int i = 0; i < src.Count; i++)
			{
				list.Add(src[i] > this.Minimum);
			}
			return list;
		}

		// Token: 0x1700024B RID: 587
		[HandlerParameter(true, "0", Min = "0", Max = "200", Step = "5")]
		public double Minimum
		{
			// Token: 0x060006A0 RID: 1696 RVA: 0x0001E294 File Offset: 0x0001C494
			get;
			// Token: 0x060006A1 RID: 1697 RVA: 0x0001E29C File Offset: 0x0001C49C
			set;
		}
	}
}
