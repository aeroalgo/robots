using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200002E RID: 46
	[HandlerCategory("vvIndicators"), HandlerName("Heiken Ashi Prices")]
	public class HeikenAshiMA : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001A4 RID: 420 RVA: 0x00007FA0 File Offset: 0x000061A0
		public IList<double> Execute(ISecurity src)
		{
			List<Bar> list = new List<Bar>(src.get_Bars().Count);
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> openPrices = src.get_OpenPrices();
			new List<Bar>(list.Count);
			List<double> list2 = new List<double>(closePrices.Count);
			List<double> list3 = new List<double>(closePrices.Count);
			List<double> list4 = new List<double>(closePrices.Count);
			List<double> list5 = new List<double>(closePrices.Count);
			for (int i = 0; i < closePrices.Count; i++)
			{
				double num;
				double num2;
				double item;
				double item2;
				if (i < 2)
				{
					num = openPrices[i];
					num2 = closePrices[i];
					item = lowPrices[i];
					item2 = highPrices[i];
				}
				else
				{
					num2 = (closePrices[i] + openPrices[i] + lowPrices[i] + highPrices[i]) / 4.0;
					num = (list5[i - 1] + list2[i - 1]) / 2.0;
					item = Math.Min(lowPrices[i], Math.Min(num, num2));
					item2 = Math.Max(highPrices[i], Math.Max(num, num2));
				}
				list5.Add(num);
				list2.Add(num2);
				list4.Add(item);
				list3.Add(item2);
			}
			if (this.Highs)
			{
				return list3;
			}
			if (this.Lows)
			{
				return list4;
			}
			if (this.Opens)
			{
				return list5;
			}
			return list2;
		}

		// Token: 0x17000089 RID: 137
		[HandlerParameter(true, "true", NotOptimized = true)]
		public bool Closess
		{
			// Token: 0x0600019C RID: 412 RVA: 0x00007F5B File Offset: 0x0000615B
			get;
			// Token: 0x0600019D RID: 413 RVA: 0x00007F63 File Offset: 0x00006163
			set;
		}

		// Token: 0x1700008D RID: 141
		public IContext Context
		{
			// Token: 0x060001A5 RID: 421 RVA: 0x00008133 File Offset: 0x00006333
			get;
			// Token: 0x060001A6 RID: 422 RVA: 0x0000813B File Offset: 0x0000633B
			set;
		}

		// Token: 0x1700008A RID: 138
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Highs
		{
			// Token: 0x0600019E RID: 414 RVA: 0x00007F6C File Offset: 0x0000616C
			get;
			// Token: 0x0600019F RID: 415 RVA: 0x00007F74 File Offset: 0x00006174
			set;
		}

		// Token: 0x1700008B RID: 139
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Lows
		{
			// Token: 0x060001A0 RID: 416 RVA: 0x00007F7D File Offset: 0x0000617D
			get;
			// Token: 0x060001A1 RID: 417 RVA: 0x00007F85 File Offset: 0x00006185
			set;
		}

		// Token: 0x1700008C RID: 140
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Opens
		{
			// Token: 0x060001A2 RID: 418 RVA: 0x00007F8E File Offset: 0x0000618E
			get;
			// Token: 0x060001A3 RID: 419 RVA: 0x00007F96 File Offset: 0x00006196
			set;
		}
	}
}
