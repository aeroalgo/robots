using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200014E RID: 334
	[HandlerCategory("vvMACD"), HandlerName("OsMA")]
	public class OsMA : MACDBase, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000A61 RID: 2657 RVA: 0x0002AFC0 File Offset: 0x000291C0
		public IList<double> Execute(IList<double> src)
		{
			IList<double> list = new List<double>(src.Count);
			IList<double> list2 = base.CalcMACD(src, this.MACDPeriod1, this.MACDPeriod2);
			IList<double> list3 = Series.EMA(list2, this.MACDSignalPeriod);
			for (int i = 0; i < src.Count; i++)
			{
				list.Add(list2[i] - list3[i]);
			}
			return list;
		}

		// Token: 0x1700036A RID: 874
		[HandlerParameter(true, "12", Min = "5", Max = "40", Step = "1")]
		public int MACDPeriod1
		{
			// Token: 0x06000A5D RID: 2653 RVA: 0x0002AF9D File Offset: 0x0002919D
			get;
			// Token: 0x06000A5E RID: 2654 RVA: 0x0002AFA5 File Offset: 0x000291A5
			set;
		}

		// Token: 0x17000369 RID: 873
		[HandlerParameter(true, "26", Min = "10", Max = "40", Step = "1")]
		public int MACDPeriod2
		{
			// Token: 0x06000A5B RID: 2651 RVA: 0x0002AF8C File Offset: 0x0002918C
			get;
			// Token: 0x06000A5C RID: 2652 RVA: 0x0002AF94 File Offset: 0x00029194
			set;
		}

		// Token: 0x1700036B RID: 875
		[HandlerParameter(true, "9", Min = "3", Max = "20", Step = "1")]
		public int MACDSignalPeriod
		{
			// Token: 0x06000A5F RID: 2655 RVA: 0x0002AFAE File Offset: 0x000291AE
			get;
			// Token: 0x06000A60 RID: 2656 RVA: 0x0002AFB6 File Offset: 0x000291B6
			set;
		}
	}
}
