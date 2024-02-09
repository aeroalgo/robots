using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000066 RID: 102
	[HandlerCategory("vvIndicators"), HandlerName("TTM Trend")]
	public class TTMTrend : IBar2BarHandler, IOneSourceHandler, IStreamHandler, IHandler, ISecurityReturns, ISecurityInputs, IContextUses
	{
		// Token: 0x06000396 RID: 918 RVA: 0x000140F0 File Offset: 0x000122F0
		public ISecurity Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> High = src.get_HighPrices();
			IList<double> Low = src.get_LowPrices();
			IList<double> openPrices = src.get_OpenPrices();
			List<Bar> list = new List<Bar>(count);
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				this.TTMperiod.ToString(),
				Low.GetHashCode().ToString()
			}, () => EMA.EMA_TSLab(Low, this.TTMperiod));
			IList<double> data2 = this.Context.GetData("ema", new string[]
			{
				this.TTMperiod.ToString(),
				High.GetHashCode().ToString()
			}, () => EMA.EMA_TSLab(High, this.TTMperiod));
			for (int i = 0; i < count; i++)
			{
				Bar bar = src.get_Bars()[i];
				double num = (data2[i] - data[i]) / 3.0 + data[i];
				double num2 = 2.0 * (data2[i] - data[i]) / 3.0 + data[i];
				double num3 = High[i];
				double num4 = Low[i];
				double arg_1AB_0 = High[i];
				double arg_1BA_0 = Low[i];
				if (closePrices[i] > num2)
				{
					num3 = High[i];
					num4 = Low[i];
					Math.Max(closePrices[i], openPrices[i]);
					Math.Min(closePrices[i], openPrices[i]);
				}
				else if (closePrices[i] < num)
				{
					num4 = High[i];
					num3 = Low[i];
					double arg_249_0 = openPrices[i];
					double arg_252_0 = closePrices[i];
				}
				Bar bar2 = new Bar(bar.get_Color(), bar.get_Date(), num4, num3, num4, num3, bar.get_Volume());
				bar2.set_Ask(bar.get_Ask());
				bar2.set_Bid(bar.get_Bid());
				bar2.set_AskQty(bar.get_AskQty());
				bar2.set_BidQty(bar.get_BidQty());
				Bar item = bar2;
				list.Add(item);
			}
			return src.CloneAndReplaceBars(list);
		}

		// Token: 0x17000134 RID: 308
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool ClassicCandles
		{
			// Token: 0x06000394 RID: 916 RVA: 0x000140A6 File Offset: 0x000122A6
			get;
			// Token: 0x06000395 RID: 917 RVA: 0x000140AE File Offset: 0x000122AE
			set;
		}

		// Token: 0x17000135 RID: 309
		public IContext Context
		{
			// Token: 0x06000397 RID: 919 RVA: 0x000143CD File Offset: 0x000125CD
			get;
			// Token: 0x06000398 RID: 920 RVA: 0x000143D5 File Offset: 0x000125D5
			set;
		}

		// Token: 0x17000133 RID: 307
		[HandlerParameter(true, "10", Min = "6", Max = "100", Step = "1")]
		public int TTMperiod
		{
			// Token: 0x06000392 RID: 914 RVA: 0x00014095 File Offset: 0x00012295
			get;
			// Token: 0x06000393 RID: 915 RVA: 0x0001409D File Offset: 0x0001229D
			set;
		}
	}
}
