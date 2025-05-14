using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000070 RID: 112
	[HandlerCategory("vvIndicators"), HandlerName("xPrice")]
	public class xPrice : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060003F9 RID: 1017 RVA: 0x00015560 File Offset: 0x00013760
		public IList<double> Execute(ISecurity sec)
		{
			int pt = 0;
			if (this.Close)
			{
				pt = 0;
			}
			if (this.Open)
			{
				pt = 1;
			}
			if (this.Low)
			{
				pt = 2;
			}
			if (this.High)
			{
				pt = 3;
			}
			if (this.MedianPrice)
			{
				pt = 4;
			}
			if (this.TypicalPrice)
			{
				pt = 5;
			}
			if (this.WeightedClose)
			{
				pt = 6;
			}
			return this.Context.GetData("xPrice", new string[]
			{
				sec.get_CacheName(),
				pt.ToString()
			}, () => xPrice.GenXPriceNum(sec, pt));
		}

		// Token: 0x060003F8 RID: 1016 RVA: 0x00015468 File Offset: 0x00013668
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
				return Series.MedianPrice(src.get_Bars());
			}
			if (_price == 5)
			{
				return Series.TypicalPrice(src.get_Bars());
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

		// Token: 0x17000151 RID: 337
		[HandlerParameter(true, "true", NotOptimized = true)]
		public bool Close
		{
			// Token: 0x060003EA RID: 1002 RVA: 0x000153EF File Offset: 0x000135EF
			get;
			// Token: 0x060003EB RID: 1003 RVA: 0x000153F7 File Offset: 0x000135F7
			set;
		}

		// Token: 0x17000158 RID: 344
		public IContext Context
		{
			// Token: 0x060003FA RID: 1018 RVA: 0x00015631 File Offset: 0x00013831
			get;
			// Token: 0x060003FB RID: 1019 RVA: 0x00015639 File Offset: 0x00013839
			set;
		}

		// Token: 0x17000154 RID: 340
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool High
		{
			// Token: 0x060003F0 RID: 1008 RVA: 0x00015422 File Offset: 0x00013622
			get;
			// Token: 0x060003F1 RID: 1009 RVA: 0x0001542A File Offset: 0x0001362A
			set;
		}

		// Token: 0x17000153 RID: 339
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Low
		{
			// Token: 0x060003EE RID: 1006 RVA: 0x00015411 File Offset: 0x00013611
			get;
			// Token: 0x060003EF RID: 1007 RVA: 0x00015419 File Offset: 0x00013619
			set;
		}

		// Token: 0x17000155 RID: 341
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool MedianPrice
		{
			// Token: 0x060003F2 RID: 1010 RVA: 0x00015433 File Offset: 0x00013633
			get;
			// Token: 0x060003F3 RID: 1011 RVA: 0x0001543B File Offset: 0x0001363B
			set;
		}

		// Token: 0x17000152 RID: 338
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Open
		{
			// Token: 0x060003EC RID: 1004 RVA: 0x00015400 File Offset: 0x00013600
			get;
			// Token: 0x060003ED RID: 1005 RVA: 0x00015408 File Offset: 0x00013608
			set;
		}

		// Token: 0x17000156 RID: 342
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool TypicalPrice
		{
			// Token: 0x060003F4 RID: 1012 RVA: 0x00015444 File Offset: 0x00013644
			get;
			// Token: 0x060003F5 RID: 1013 RVA: 0x0001544C File Offset: 0x0001364C
			set;
		}

		// Token: 0x17000157 RID: 343
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool WeightedClose
		{
			// Token: 0x060003F6 RID: 1014 RVA: 0x00015455 File Offset: 0x00013655
			get;
			// Token: 0x060003F7 RID: 1015 RVA: 0x0001545D File Offset: 0x0001365D
			set;
		}
	}
}
