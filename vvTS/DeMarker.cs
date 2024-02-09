using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x020001AD RID: 429
	[HandlerCategory("vvIndicators"), HandlerName("DeMarker")]
	public class DeMarker : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000D9C RID: 3484 RVA: 0x0003B6C8 File Offset: 0x000398C8
		public IList<double> Execute(ISecurity _sec)
		{
			IList<double> highPrices = _sec.get_HighPrices();
			IList<double> lowPrices = _sec.get_LowPrices();
			IList<double> closePrices = _sec.get_ClosePrices();
			List<double> DeMax = new List<double>(closePrices.Count);
			List<double> DeMin = new List<double>(closePrices.Count);
			List<double> list = new List<double>(closePrices.Count);
			DeMax.Add(0.0);
			DeMin.Add(0.0);
			for (int i = 1; i < closePrices.Count; i++)
			{
				double item;
				if (highPrices[i] > highPrices[i - 1])
				{
					item = highPrices[i] - highPrices[i - 1];
				}
				else
				{
					item = 0.0;
				}
				double item2;
				if (lowPrices[i] < lowPrices[i - 1])
				{
					item2 = lowPrices[i - 1] - lowPrices[i];
				}
				else
				{
					item2 = 0.0;
				}
				DeMax.Add(item);
				DeMin.Add(item2);
			}
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				this.Period.ToString(),
				DeMax.GetHashCode().ToString()
			}, () => Series.SMA(DeMax, this.Period));
			IList<double> data2 = this.Context.GetData("sma", new string[]
			{
				this.Period.ToString(),
				DeMin.GetHashCode().ToString()
			}, () => Series.SMA(DeMin, this.Period));
			for (int j = 0; j < closePrices.Count; j++)
			{
				double item3;
				if (j < this.Period)
				{
					item3 = 0.0;
				}
				else
				{
					item3 = data[j] / (data[j] + data2[j]);
				}
				list.Add(item3);
			}
			IList<double> result = list;
			if (this.postSmooth > 0)
			{
				result = JMA.GenJMA(list, this.postSmooth, 100);
			}
			return result;
		}

		// Token: 0x1700046C RID: 1132
		public IContext Context
		{
			// Token: 0x06000D9D RID: 3485 RVA: 0x0003B932 File Offset: 0x00039B32
			get;
			// Token: 0x06000D9E RID: 3486 RVA: 0x0003B93A File Offset: 0x00039B3A
			set;
		}

		// Token: 0x1700046A RID: 1130
		[HandlerParameter(true, "15", Min = "1", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x06000D98 RID: 3480 RVA: 0x0003B66B File Offset: 0x0003986B
			get;
			// Token: 0x06000D99 RID: 3481 RVA: 0x0003B673 File Offset: 0x00039873
			set;
		}

		// Token: 0x1700046B RID: 1131
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x06000D9A RID: 3482 RVA: 0x0003B67C File Offset: 0x0003987C
			get;
			// Token: 0x06000D9B RID: 3483 RVA: 0x0003B684 File Offset: 0x00039884
			set;
		}
	}
}
