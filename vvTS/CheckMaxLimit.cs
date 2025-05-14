using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B8 RID: 184
	[HandlerCategory("vvTrade"), HandlerName("Меньше ли лимита")]
	public class CheckMaxLimit : IDoubleCompaper1Handler, IOneSourceHandler, IBooleanReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x0600069E RID: 1694 RVA: 0x0001E24C File Offset: 0x0001C44C
		public IList<bool> Execute(IList<double> src)
		{
			List<bool> list = new List<bool>(src.Count);
			for (int i = 0; i < src.Count; i++)
			{
				list.Add(src[i] < this.Limit);
			}
			return list;
		}

		// Token: 0x1700024A RID: 586
		[HandlerParameter(true, "0", Min = "0", Max = "200", Step = "5")]
		public double Limit
		{
			// Token: 0x0600069C RID: 1692 RVA: 0x0001E239 File Offset: 0x0001C439
			get;
			// Token: 0x0600069D RID: 1693 RVA: 0x0001E241 File Offset: 0x0001C441
			set;
		}
	}
}
