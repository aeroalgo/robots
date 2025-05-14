using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000158 RID: 344
	[HandlerCategory("vvAverages"), HandlerName("AMAext")]
	public class AMAext : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000AD0 RID: 2768 RVA: 0x0002CADC File Offset: 0x0002ACDC
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("amaext", new string[]
			{
				this.AMAPeriod.ToString(),
				this.nfast.ToString(),
				this.nslow.ToString(),
				src.GetHashCode().ToString()
			}, () => AMAext.GenAMAext(src, this.AMAPeriod, this.nfast, this.nslow));
		}

		// Token: 0x06000ACF RID: 2767 RVA: 0x0002C98C File Offset: 0x0002AB8C
		public static IList<double> GenAMAext(IList<double> src, int amaperiod, int _nfast, int _nslow)
		{
			int num = amaperiod + 2;
			int count = src.Count;
			if (count < num)
			{
				return null;
			}
			double[] array = new double[count];
			double y = 2.0;
			double num2 = 2.0 / (double)(_nslow + 1);
			double num3 = 2.0 / (double)(_nfast + 1);
			double num4 = src[0];
			for (int i = amaperiod + 2; i < count; i++)
			{
				double num5 = Math.Abs(src[i] - src[i - amaperiod]);
				double num6 = 1E-09;
				for (int j = 0; j < amaperiod; j++)
				{
					num6 += Math.Abs(src[i - j] - src[i - j - 1]);
				}
				double num7 = num5 / num6;
				double num8 = num3 - num2;
				double num9 = num7 * num8;
				double x = num9 + num2;
				double num10 = num4 + Math.Pow(x, y) * (src[i] - num4);
				array[i] = num10;
				num4 = num10;
			}
			return array;
		}

		// Token: 0x17000392 RID: 914
		[HandlerParameter(true, "20", Min = "10", Max = "100", Step = "5")]
		public int AMAPeriod
		{
			// Token: 0x06000AC9 RID: 2761 RVA: 0x0002C959 File Offset: 0x0002AB59
			get;
			// Token: 0x06000ACA RID: 2762 RVA: 0x0002C961 File Offset: 0x0002AB61
			set;
		}

		// Token: 0x17000395 RID: 917
		public IContext Context
		{
			// Token: 0x06000AD1 RID: 2769 RVA: 0x0002CB6F File Offset: 0x0002AD6F
			get;
			// Token: 0x06000AD2 RID: 2770 RVA: 0x0002CB77 File Offset: 0x0002AD77
			set;
		}

		// Token: 0x17000393 RID: 915
		[HandlerParameter(true, "2", Min = "1", Max = "10", Step = "1")]
		public int nfast
		{
			// Token: 0x06000ACB RID: 2763 RVA: 0x0002C96A File Offset: 0x0002AB6A
			get;
			// Token: 0x06000ACC RID: 2764 RVA: 0x0002C972 File Offset: 0x0002AB72
			set;
		}

		// Token: 0x17000394 RID: 916
		[HandlerParameter(true, "30", Min = "1", Max = "50", Step = "1")]
		public int nslow
		{
			// Token: 0x06000ACD RID: 2765 RVA: 0x0002C97B File Offset: 0x0002AB7B
			get;
			// Token: 0x06000ACE RID: 2766 RVA: 0x0002C983 File Offset: 0x0002AB83
			set;
		}
	}
}
