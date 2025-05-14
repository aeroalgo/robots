using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000071 RID: 113
	[HandlerCategory("vvIndicators"), HandlerName("xPrice2")]
	public class xPrice2 : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000400 RID: 1024 RVA: 0x0001575C File Offset: 0x0001395C
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("xPrice", new string[]
			{
				sec.get_CacheName(),
				this.Price.ToString()
			}, () => xPrice.GenXPriceNum(sec, this.Price));
		}

		// Token: 0x060003FF RID: 1023 RVA: 0x0001565C File Offset: 0x0001385C
		public static IList<double> GenXPriceNum(ISecurity src, int _price)
		{
			int count = src.get_Bars().Count;
			if (_price == 0)
			{
				return src.get_ClosePrices();
			}
			if (_price == 1)
			{
				return src.get_OpenPrices();
			}
			if (_price == 2)
			{
				return src.get_LowPrices();
			}
			if (_price == 3)
			{
				return src.get_HighPrices();
			}
			if (_price == 4)
			{
				return vvSeries.MedianPrice(src.get_Bars());
			}
			if (_price == 5)
			{
				return vvSeries.TypicalPrice(src.get_Bars());
			}
			if (_price == 6)
			{
				IList<double> closePrices = src.get_ClosePrices();
				IList<double> arg_68_0 = src.get_OpenPrices();
				IList<double> lowPrices = src.get_LowPrices();
				IList<double> highPrices = src.get_HighPrices();
				double[] array = new double[count];
				for (int i = 0; i < closePrices.Count; i++)
				{
					array[i] = (closePrices[i] * 2.0 + highPrices[i] + lowPrices[i]) / 4.0;
				}
				return array;
			}
			return src.get_ClosePrices();
		}

		// Token: 0x1700015A RID: 346
		public IContext Context
		{
			// Token: 0x06000401 RID: 1025 RVA: 0x000157C0 File Offset: 0x000139C0
			get;
			// Token: 0x06000402 RID: 1026 RVA: 0x000157C8 File Offset: 0x000139C8
			set;
		}

		// Token: 0x17000159 RID: 345
		[HandlerParameter(true, "true", NotOptimized = true)]
		public int Price
		{
			// Token: 0x060003FD RID: 1021 RVA: 0x0001564A File Offset: 0x0001384A
			get;
			// Token: 0x060003FE RID: 1022 RVA: 0x00015652 File Offset: 0x00013852
			set;
		}
	}
}
